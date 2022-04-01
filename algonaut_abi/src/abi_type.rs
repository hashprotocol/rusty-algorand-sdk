use crate::error::AbiError;
use lazy_static::lazy_static;
use regex::Regex;
use std::{convert::TryInto, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BaseType(u32);

/// Uint is the index (0) for `Uint` type in ABI encoding.
const UINT: u32 = 0;

/// Byte is the index (1) for `Byte` type in ABI encoding.
const BYTE: u32 = 1;

/// Ufixed is the index (2) for `UFixed` type in ABI encoding.
const UFIXED: u32 = 2;

/// Bool is the index (3) for `Bool` type in ABI encoding.
const BOOL: u32 = 3;

/// ArrayStatic is the index (4) for static length array (<type>[length]) type in ABI encoding.
const ARRAY_STATIC: u32 = 4;

/// Address is the index (5) for `Address` type in ABI encoding (an type alias of Byte[32]).
const ADDRESS: u32 = 5;

/// ArrayDynamic is the index (6) for dynamic length array (<type>[]) type in ABI encoding.
const ARRAY_DYNAMIC: u32 = 6;

/// String is the index (7) for `String` type in ABI encoding (an type alias of Byte[]).
const STRING: u32 = 7;

/// Tuple is the index (8) for tuple `(<type 0>, ..., <type k>)` in ABI encoding.
const TUPLE: u32 = 8;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Type is the struct that stores information about an ABI value's type.
pub struct AbiType {
    abi_type_id: BaseType,
    child_types: Vec<AbiType>,

    /// only can be applied to `uint` bitSize <N> or `ufixed` bitSize <N>
    bit_size: Option<u16>,
    /// only can be applied to `ufixed` precision <M>,
    precision: Option<u16>,

    // length for static array / tuple
    /*
        by ABI spec, len over binary array returns number of bytes
        the type is uint16, which allows for only lenth in [0, 2^16 - 1]
        representation of static length can only be constrained in uint16 type
    */
    /// NOTE may want to change back to uint32/uint64
    static_length: Option<u16>,
}

impl AbiType {
    /// Serialize an ABI Type to a string in ABI encoding.
    pub fn string(&self) -> Result<String, AbiError> {
        match (
            self.abi_type_id.0,
            self.bit_size,
            self.precision,
            self.static_length,
        ) {
            (UINT, Some(bit_size), None, None) if self.child_types.is_empty() => {
                Ok(format!("uint{}", bit_size))
            }
            (BYTE, None, None, None) if self.child_types.is_empty() => Ok("byte".to_owned()),
            (UFIXED, Some(bit_size), Some(precision), None) if self.child_types.is_empty() => {
                Ok(format!("ufixed{}{}", bit_size, precision))
            }
            (BOOL, None, None, None) if self.child_types.is_empty() => Ok("bool".to_owned()),
            (ARRAY_STATIC, None, None, Some(static_length)) if !self.child_types.is_empty() => Ok(
                format!("{}[{}]", self.child_types[0].string()?, static_length),
            ),
            (ARRAY_DYNAMIC, None, None, None) if !self.child_types.is_empty() => {
                Ok(format!("{}[]", self.child_types[0].string()?))
            }
            (STRING, None, None, None) if self.child_types.is_empty() => Ok("string".to_owned()),
            (TUPLE, None, None, None) => {
                let mut type_strings = Vec::with_capacity(self.child_types.len());
                for child_type in &self.child_types {
                    type_strings.push(child_type.string()?)
                }
                Ok(format!("({})", type_strings.join(",")))
            }
            _ => Err(AbiError::Msg(
                "Invalid state: not serializable abi type state: {self:?}".to_owned(),
            )),
        }
    }
}

impl FromStr for AbiType {
    type Err = AbiError;

    /// Parses an ABI type string.
    /// For example: `TypeOf("(uint64,byte[])")`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(stripped) = s.strip_suffix("[]") {
            let array_arg_type = stripped.parse()?;
            Ok(make_dynamic_array_type(array_arg_type))
        } else if s.ends_with(']') {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
            }
            let caps = RE.captures(s).unwrap();

            if caps.len() != 3 {
                return Err(AbiError::Msg(format!("ill formed uint type: {s}")));
            }
            let array_type = caps[1].parse()?;
            let array_len_s = caps[2].to_owned();

            let array_len: usize = array_len_s.parse().map_err(|e| {
                AbiError::Msg(format!("Error parsing array len: {array_len_s}: {e:?}"))
            })?;

            Ok(make_static_array_type(
                array_type,
                array_len.try_into().map_err(|_| {
                    AbiError::Msg("Couldn't convert array_len: {array_len} in u16".to_owned())
                })?,
            ))
        } else if let Some(stripped) = s.strip_prefix("uint") {
            let type_size = stripped
                .parse()
                .map_err(|e| AbiError::Msg(format!("Ill formed uint type: {s}: {e:?}")))?;

            make_uint_type(type_size)
        } else if s == "byte" {
            Ok(make_empty_fields_type(BaseType(BYTE)))
        } else if s.starts_with("ufixed") {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"^ufixed([1-9][\d]*)x([1-9][\d]*)$").unwrap();
            }
            let caps = RE.captures(s).unwrap();

            if caps.len() != 3 {
                return Err(AbiError::Msg(format!("ill formed ufixed type: {s}")));
            }
            let ufixed_size_s = &caps[1].to_owned();
            let ufixed_size = ufixed_size_s.parse().map_err(|e| {
                AbiError::Msg(format!("Error parsing ufixed size: {ufixed_size_s}: {e:?}"))
            })?;

            let ufixed_precision_s = &caps[2].to_owned();
            let ufixed_precision = ufixed_precision_s.parse().map_err(|e| {
                AbiError::Msg(format!(
                    "Error parsing ufixed precision: {ufixed_precision_s}: {e:?}"
                ))
            })?;

            make_ufixed_type(ufixed_size, ufixed_precision)
        } else if s == "bool" {
            Ok(make_empty_fields_type(BaseType(BOOL)))
        } else if s == "address" {
            Ok(make_empty_fields_type(BaseType(ADDRESS)))
        } else if s == "string" {
            Ok(make_empty_fields_type(BaseType(STRING)))
        } else if s.len() >= 2 && s.starts_with('(') && s.ends_with(')') {
            let tuple_content = parse_tuple_content(&s[1..s.len() - 1])?;
            let mut tuple_types = Vec::with_capacity(tuple_content.len());

            for t in tuple_content {
                let ti = t.parse()?;
                tuple_types.push(ti);
            }

            make_tuple_type(tuple_types)
        } else {
            Err(AbiError::Msg(format!(
                "cannot convert string: {s} to ABI type"
            )))
        }
    }
}

fn make_dynamic_array_type(arg_type: AbiType) -> AbiType {
    AbiType {
        abi_type_id: BaseType(ARRAY_DYNAMIC),
        child_types: vec![arg_type],
        bit_size: None,
        precision: None,
        static_length: None,
    }
}

fn make_static_array_type(arg_type: AbiType, array_len: u16) -> AbiType {
    AbiType {
        abi_type_id: BaseType(ARRAY_STATIC),
        child_types: vec![arg_type],
        bit_size: None,
        precision: None,
        static_length: Some(array_len),
    }
}

/// Makes `Uint` ABI type by taking a type bitSize argument.
/// The range of type bitSize is [8, 512] and type bitSize % 8 == 0.
fn make_uint_type(type_size: i32) -> Result<AbiType, AbiError> {
    if type_size % 8 != 0 || type_size < 8 || type_size > 512 {
        return Err(AbiError::Msg(format!(
            "unsupported uint type bitSize: {type_size}"
        )));
    }
    Ok(AbiType {
        abi_type_id: BaseType(UINT),
        child_types: vec![],
        bit_size: Some(type_size as u16),
        precision: None,
        static_length: None,
    })
}

fn make_empty_fields_type(type_: BaseType) -> AbiType {
    AbiType {
        abi_type_id: type_,
        child_types: vec![],
        bit_size: None,
        precision: None,
        static_length: None,
    }
}

/// Makes `UFixed` ABI type by taking type bitSize and type precision as arguments.
/// The range of type bitSize is [8, 512] and type bitSize % 8 == 0.
/// The range of type precision is [1, 160].
fn make_ufixed_type(type_size: u32, type_precision: u32) -> Result<AbiType, AbiError> {
    if type_size % 8 != 0 || !(8..=512).contains(&type_size) {
        return Err(AbiError::Msg(format!(
            "unsupported ufixed type bitSize: {type_size}"
        )));
    }
    if !(1..=160).contains(&type_precision) {
        return Err(AbiError::Msg(format!(
            "unsupported ufixed type precision: {type_precision}"
        )));
    }
    Ok(AbiType {
        abi_type_id: BaseType(UFIXED),
        child_types: vec![],
        bit_size: Some(type_size as u16), // cast: safe bounds checked in this fn
        precision: Some(type_precision as u16), // cast: safe bounds checked in this fn
        static_length: None,
    })
}

/// Makes tuple ABI type by taking an array of tuple element types as argument.
fn make_tuple_type(argument_types: Vec<AbiType>) -> Result<AbiType, AbiError> {
    if argument_types.len() >= u16::MAX as usize {
        return Err(AbiError::Msg(
            "tuple type child type number larger than maximum uint16 error".to_owned(),
        ));
    }

    Ok(AbiType {
        abi_type_id: BaseType(TUPLE),
        static_length: Some(argument_types.len() as u16), // cast: safe bounds checked in this fn
        child_types: argument_types,
        bit_size: None,
        precision: None,
    })
}

/// Keeps track of the start and end of a segment in a string.
struct Segment {
    left: usize,
    right: usize,
}

/// Splits an ABI encoded string for tuple type into multiple sub-strings.
/// Each sub-string represents a content type of the tuple type.
/// The argument str is the content between parentheses of tuple, i.e.
/// (...... str ......)
///  ^               ^
fn parse_tuple_content(str: &str) -> Result<Vec<String>, AbiError> {
    // if the tuple type content is empty (which is also allowed)
    // just return the empty string list
    if str.is_empty() {
        return Ok(vec![]);
    }

    // the following 2 checks want to make sure input string can be separated by comma
    // with form: "...substr_0,...substr_1,...,...substr_k"

    // str should not have leading/tailing comma
    if str.ends_with(',') || str.starts_with(',') {
        return Err(AbiError::Msg(
            "parsing error: tuple content should not start with comma".to_owned(),
        ));
    }
    // str should not have consecutive commas
    if str.contains(",,") {
        return Err(AbiError::Msg("no consecutive commas".to_owned()));
    }

    let mut paren_segment_record = vec![];
    let mut stack = vec![];

    // get the most exterior parentheses segment (not overlapped by other parentheses)
    // illustration: "*****,(*****),*****" => ["*****", "(*****)", "*****"]
    // once iterate to left paren (, stack up by 1 in stack
    // iterate to right paren ), pop 1 in stack
    // if iterate to right paren ) with stack height 0, find a parenthesis segment "(******)"
    for (index, chr) in str.chars().enumerate() {
        if chr == '(' {
            stack.push(index);
        } else if chr == ')' {
            if stack.is_empty() {
                return Err(AbiError::Msg(format!("unpaired parentheses: {str}")));
            }

            let left_paren_index = stack[stack.len() - 1];
            stack.pop();
            if stack.is_empty() {
                paren_segment_record.push(Segment {
                    left: left_paren_index,
                    right: index,
                });
            }
        }
    }
    if !stack.is_empty() {
        return Err(AbiError::Msg(format!("unpaired parentheses: {str}")));
    }

    // take out tuple-formed type str in tuple argument
    let mut str_copied = str.to_owned();

    for paren_seg in paren_segment_record.iter().rev() {
        str_copied = format!(
            "{}{}",
            str_copied[..paren_seg.left].to_owned(),
            str_copied[paren_seg.right + 1..].to_owned()
        );
    }

    // split the string without parenthesis segments
    let tuple_str_segs: Vec<&str> = str_copied.split(',').collect();
    let mut tuple_str_segs_res: Vec<String> = Vec::with_capacity(tuple_str_segs.len());

    // the empty strings are placeholders for parenthesis segments
    // put the parenthesis segments back into segment list
    let mut paren_seg_count = 0;
    for seg_str in tuple_str_segs.iter() {
        if seg_str.is_empty() {
            let paren_seg = &paren_segment_record[paren_seg_count];
            tuple_str_segs_res.push(str[paren_seg.left..paren_seg.right + 1].to_owned());
            paren_seg_count += 1;
        } else {
            tuple_str_segs_res.push((*seg_str).to_owned());
        }
    }

    Ok(tuple_str_segs_res)
}
