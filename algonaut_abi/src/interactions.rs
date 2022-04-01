use super::abi_type::AbiType;
use crate::error::AbiError;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::{collections::HashMap, convert::TryInto};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TransactionArgType {
    Any,
    Payment,
    KeyRegistration,
    AssetConfig,
    AssetTransfer,
    AssetFreeze,
    AppCall,
}

impl TransactionArgType {
    fn from_api_str(s: &str) -> Result<TransactionArgType, AbiError> {
        match s {
            "Any" => Ok(TransactionArgType::Any),
            "Payment" => Ok(TransactionArgType::Payment),
            "KeyRegistration" => Ok(TransactionArgType::KeyRegistration),
            "AssetConfig" => Ok(TransactionArgType::AssetConfig),
            "AssetTransfer" => Ok(TransactionArgType::AssetTransfer),
            "AssetFreeze" => Ok(TransactionArgType::AssetFreeze),
            "AppCall" => Ok(TransactionArgType::AppCall),
            _ => Err(AbiError::Msg(format!(
                "Not supported transaction arg type api string: {s}"
            ))),
        }
    }

    fn is_valid_str(s: &str) -> bool {
        Self::from_api_str(s).is_ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReferenceArgType {
    Account,
    Asset,
    Application,
}

impl ReferenceArgType {
    fn from_api_str(s: &str) -> Result<ReferenceArgType, AbiError> {
        match s {
            "AccountReferenceType" => Ok(ReferenceArgType::Account),
            "AssetReferenceType" => Ok(ReferenceArgType::Asset),
            "ApplicationReferenceType" => Ok(ReferenceArgType::Application),
            _ => Err(AbiError::Msg(format!(
                "Not supported reference arg type api string: {s}"
            ))),
        }
    }

    fn is_valid_str(s: &str) -> bool {
        Self::from_api_str(s).is_ok()
    }
}

/// Represents an ABI Method argument
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AbiArg {
    /// User-friendly name for the argument
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The type of the argument as a string.
    /// See [get_type_object](get_type_object) to obtain the ABI type object
    #[serde(rename = "type")]
    pub type_: String,

    /// User-friendly description for the argument
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Cache that holds the parsed type object
    #[serde(skip)]
    pub parsed: Option<AbiType>,
}

impl AbiArg {
    fn is_transaction_arg(&self) -> bool {
        TransactionArgType::is_valid_str(&self.type_)
    }

    fn is_reference_arg(&self) -> bool {
        ReferenceArgType::is_valid_str(&self.type_)
    }

    /// parses and returns the ABI type object for this argument's
    /// type. An error will be returned if this argument's type is a transaction or
    /// reference type
    fn get_type_object(&mut self) -> Result<AbiType, AbiError> {
        if self.is_transaction_arg() {
            return Err(AbiError::Msg(format!(
                "Invalid operation on transaction type: {}",
                self.type_
            )));
        }
        if self.is_reference_arg() {
            return Err(AbiError::Msg(format!(
                "Invalid operation on reference type: {}",
                self.type_
            )));
        }
        if let Some(parsed) = &self.parsed {
            return Ok(parsed.clone());
        }

        let type_obj = self.type_.parse::<AbiType>()?;
        self.parsed = Some(type_obj.clone());

        Ok(type_obj)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Represents an ABI method return value
pub struct AbiReturn {
    /// The type of the argument as a string. See the [get_type_object](get_type_object) to
    /// obtain the ABI type object
    #[serde(rename = "type")]
    pub type_: String,

    /// User-friendly description for the argument
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Cache that holds the parsed type object
    #[serde(skip)]
    pub parsed: Option<AbiType>,
}

impl AbiReturn {
    fn is_void(&self) -> bool {
        self.type_ == "void"
    }

    fn get_type_object(&mut self) -> Result<AbiType, AbiError> {
        if self.is_void() {
            return Err(AbiError::Msg(
                "Invalid operation on void return type".to_owned(),
            ));
        }
        if let Some(parsed) = &self.parsed {
            return Ok(parsed.clone());
        }

        let type_obj = self.type_.parse::<AbiType>()?;
        self.parsed = Some(type_obj.clone());

        Ok(type_obj)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Represents an ABI method return value
pub struct AbiMethod {
    /// The name of the method
    #[serde(rename = "name")]
    pub name: String,

    /// User-friendly description for the method
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The arguments of the method, in order
    #[serde(rename = "args", skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<AbiArg>,

    /// Information about the method's return value
    #[serde(rename = "returns")]
    pub returns: AbiReturn,
}

impl AbiMethod {
    /// Calculates and returns the signature of the method
    pub fn get_signature(&self) -> String {
        let method_signature = format!("{}{}", self.name, "(");
        let mut str_types: Vec<String> = vec![];
        for arg in &self.args {
            str_types.push(arg.type_.to_owned());
        }
        format!(
            "{method_signature}{}){}",
            str_types.join(","),
            self.returns.type_
        )
    }

    /// Calculates and returns the 4-byte selector of the method
    pub fn get_selector(&self) -> Result<[u8; 4], AbiError> {
        let sig = self.get_signature();
        let sig_hash = sha2::Sha512_256::digest(sig);
        Ok(sig_hash[..4]
            .try_into()
            .expect("Unexpected: couldn't get signature bytes from Sha512_256 digest"))
    }

    /// Returns the number of transactions required to invoke this method
    pub fn get_tx_count(&self) -> usize {
        1 + self.args.iter().filter(|a| a.is_transaction_arg()).count()
    }

    /// Decodes a method signature string into a Method object.
    pub fn from_signature(method_str: &str) -> Result<AbiMethod, AbiError> {
        let open_idx = method_str.chars().position(|c| c == '(').ok_or_else(|| {
            AbiError::Msg("method signature is missing an open parenthesis".to_owned())
        })?;

        let name = &method_str[..open_idx];
        if name.is_empty() {
            return Err(AbiError::Msg(
                "method must have a non empty name".to_owned(),
            ));
        }

        let (arg_types, close_idx) = parse_method_args(method_str, open_idx)?;

        let mut return_type = AbiReturn {
            type_: method_str[close_idx + 1..].to_owned(),
            description: None,
            parsed: None,
        };

        if !return_type.is_void() {
            // fill type object cache
            return_type.get_type_object()?;
        }

        let mut args: Vec<AbiArg> = Vec::with_capacity(arg_types.len());

        for (i, arg_type) in arg_types.into_iter().enumerate() {
            let arg = AbiArg {
                type_: arg_type.clone(),
                name: None,
                description: None,
                parsed: None,
            };
            args.push(arg);

            if TransactionArgType::is_valid_str(&arg_type)
                || ReferenceArgType::is_valid_str(&arg_type)
            {
                continue;
            }

            // fill type object cache
            args[i].get_type_object()?;
        }

        Ok(AbiMethod {
            name: name.to_owned(),
            args,
            returns: return_type,
            description: None,
        })
    }
}

/// Parses the arguments from a method signature string.
/// str_method is the complete method signature and start_idx is the index of the
/// opening parenthesis of the arguments list. This function returns a list of
/// the argument types from the method signature and the index of the closing
/// parenthesis of the arguments list.
fn parse_method_args(str_method: &str, start_idx: usize) -> Result<(Vec<String>, usize), AbiError> {
    // handle no args
    if start_idx < str_method.len() - 1 && str_method.chars().nth(start_idx + 1) == Some(')') {
        return Ok((vec![], start_idx + 1));
    }

    let mut arg_types = vec![];

    let mut paren_cnt = 1;
    let mut prev_pos = start_idx + 1;
    let mut close_idx = None;
    let init_prev_pos = prev_pos;

    for cur_pos in init_prev_pos..str_method.len() {
        let chars = str_method.chars().collect::<Vec<_>>();
        if chars[cur_pos] == '(' {
            paren_cnt += 1;
        } else if chars[cur_pos] == ')' {
            paren_cnt -= 1;
        }

        if paren_cnt < 0 {
            return Err(AbiError::Msg(
                "method signature parentheses mismatch".to_owned(),
            ));
        } else if paren_cnt > 1 {
            continue;
        }

        if chars[cur_pos] == ',' || paren_cnt == 0 {
            let str_arg = &str_method[prev_pos..cur_pos];
            arg_types.push(str_arg.to_owned());
            prev_pos = cur_pos + 1;
        }

        if paren_cnt == 0 {
            close_idx = Some(cur_pos);
            break;
        }
    }

    if let Some(close_idx) = close_idx {
        Ok((arg_types, close_idx))
    } else {
        Err(AbiError::Msg(
            "method signature parentheses mismatch".to_owned(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents an ABI interface, which is a logically grouped collection of methods
pub struct AbiInterface {
    /// The name of the interface
    #[serde(rename = "name")]
    pub name: String,

    /// User-friendly description for the interface
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The methods that the interface contains
    #[serde(rename = "methods", skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<AbiMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// ContractNetworkInfo contains network-specific information about the contract
pub struct AbiContractNetworkInfo {
    /// The application ID of the contract for this network
    #[serde(rename = "appID")]
    pub app_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Represents an ABI contract, which is a concrete set of methods implemented by a single app
pub struct AbiContract {
    /// The name of the contract
    #[serde(rename = "name")]
    pub name: String,

    /// User-friendly description for the contract
    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional information about the contract's instances across different networks
    #[serde(rename = "networks", skip_serializing_if = "HashMap::is_empty")]
    pub networks: HashMap<String, AbiContractNetworkInfo>,

    /// The methods that the interface contains
    #[serde(rename = "methods", skip_serializing_if = "Vec::is_empty")]
    pub methods: Vec<AbiMethod>,
}
