#[cfg(test)]
mod tests {
    use crate::{
        abi_type::AbiType,
        interactions::{
            AbiArg, AbiContract, AbiContractNetworkInfo, AbiInterface, AbiMethod, AbiReturn,
        },
    };

    #[test]
    fn test_method_from_signature() {
        let uint32_abi_type_res = "uint32".parse::<AbiType>();
        assert!(uint32_abi_type_res.is_ok());
        let uint32_abi_type = uint32_abi_type_res.clone().unwrap();

        let expected_args = vec![
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
        ];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type),
            },
        };

        let method_sig = "add(uint32,uint32)uint32";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_with_tuple() {
        let uint32_abi_type_res = "uint32".parse::<AbiType>();
        assert!(uint32_abi_type_res.is_ok());
        let uint32_abi_type = uint32_abi_type_res.clone().unwrap();

        let uint32_tuple_abi_type_res = "(uint32,uint32)".parse::<AbiType>();
        assert!(uint32_tuple_abi_type_res.is_ok());
        let uint32_tuple_abi_type = uint32_tuple_abi_type_res.clone().unwrap();

        let uint32_tuple_tuple_abi_type_res = "(uint32,(uint32,uint32))".parse::<AbiType>();
        assert!(uint32_tuple_tuple_abi_type_res.is_ok());
        let uint32_tuple_tuple_abi_type = uint32_tuple_tuple_abi_type_res.clone().unwrap();

        let expected_args = vec![
            AbiArg {
                name: None,
                type_: "(uint32,(uint32,uint32))".to_owned(),
                description: None,
                parsed: Some(uint32_tuple_tuple_abi_type.clone()),
            },
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
        ];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "(uint32,uint32)".to_owned(),
                description: None,
                parsed: Some(uint32_tuple_abi_type),
            },
        };

        let method_sig = "add((uint32,(uint32,uint32)),uint32)(uint32,uint32)";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_with_void_return() {
        let uint32_abi_type_res = "uint32".parse::<AbiType>();
        assert!(uint32_abi_type_res.is_ok());
        let uint32_abi_type = uint32_abi_type_res.clone().unwrap();

        let expected_args = vec![
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: Some(uint32_abi_type.clone()),
            },
        ];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "void".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let method_sig = "add(uint32,uint32)void";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_with_no_args() {
        let expected_args = vec![];
        let expected = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args: expected_args,
            returns: AbiReturn {
                type_: "void".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let method_sig = "add()void";

        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_ok());
        let method = result.unwrap();

        assert_eq!(expected, method);
    }

    #[test]
    fn test_method_from_signature_invalid_format() {
        let method_sig = "add)uint32,uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());

        let method_sig = "add(uint32, uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());

        let method_sig = "(uint32,uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());

        let method_sig = "add((uint32, uint32)uint32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_method_from_signature_invalid_abi_type() {
        let method_sig = "add(uint32,uint32)int32";
        let result = AbiMethod::from_signature(method_sig);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_signature() {
        let args = vec![
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let expected = "add(uint32,uint32)uint32";

        assert_eq!(expected, method.get_signature());
    }

    #[test]
    fn test_get_selector() {
        let args = vec![
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiArg {
                name: None,
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let expected = [0x3e, 0x1e, 0x52, 0xbd];

        let selector_res = method.get_selector();
        assert!(selector_res.is_ok());
        let selector = selector_res.unwrap();

        assert_eq!(expected, selector);
    }

    #[test]
    fn test_encode_json_method() {
        let args = vec![
            AbiArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let expected = r#"{"name":"add","args":[{"name":"0","type":"uint32"},{"name":"1","type":"uint32"}],"returns":{"type":"uint32"}}"#;

        let json_res = serde_json::to_string(&method);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_method_with_description() {
        let args = vec![
            AbiArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
            AbiArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: Some("description".to_owned()),
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        };

        let expected = r#"{"name":"add","desc":"description","args":[{"name":"0","type":"uint32","desc":"description"},{"name":"1","type":"uint32","desc":"description"}],"returns":{"type":"uint32","desc":"description"}}"#;

        let json_res = serde_json::to_string(&method);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_interface() {
        let args = vec![
            AbiArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let interface_object = AbiInterface {
            name: "interface".to_owned(),
            description: None,
            methods: vec![method],
        };

        let expected = r#"{"name":"interface","methods":[{"name":"add","args":[{"name":"0","type":"uint32"},{"name":"1","type":"uint32"}],"returns":{"type":"uint32"}}]}"#;

        let json_res = serde_json::to_string(&interface_object);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_interface_with_description() {
        let args = vec![
            AbiArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
            AbiArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: Some("description".to_owned()),
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        };

        let interface_object = AbiInterface {
            name: "interface".to_owned(),
            description: None,
            methods: vec![method],
        };

        let expected = r#"{"name":"interface","methods":[{"name":"add","desc":"description","args":[{"name":"0","type":"uint32","desc":"description"},{"name":"1","type":"uint32","desc":"description"}],"returns":{"type":"uint32","desc":"description"}}]}"#;

        let json_res = serde_json::to_string(&interface_object);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_contract() {
        let args = vec![
            AbiArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
            AbiArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: None,
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: None,
                parsed: None,
            },
        };

        let network = AbiContractNetworkInfo { app_id: 123 };

        let contract = AbiContract {
            name: "contract".to_owned(),
            networks: [("genesis hash".to_owned(), network)].into(),
            description: None,
            methods: vec![method],
        };

        let expected = r#"{"name":"contract","networks":{"genesis hash":{"appID":123}},"methods":[{"name":"add","args":[{"name":"0","type":"uint32"},{"name":"1","type":"uint32"}],"returns":{"type":"uint32"}}]}"#;

        let json_res = serde_json::to_string(&contract);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }

    #[test]
    fn test_encode_json_contract_with_description() {
        let args = vec![
            AbiArg {
                name: Some("0".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
            AbiArg {
                name: Some("1".to_owned()),
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        ];
        let method = AbiMethod {
            name: "add".to_owned(),
            description: Some("description".to_owned()),
            args,
            returns: AbiReturn {
                type_: "uint32".to_owned(),
                description: Some("description".to_owned()),
                parsed: None,
            },
        };

        let network = AbiContractNetworkInfo { app_id: 123 };

        let contract = AbiContract {
            name: "contract".to_owned(),
            networks: [("genesis hash".to_owned(), network)].into(),
            description: Some("description for contract".to_owned()),
            methods: vec![method],
        };

        let expected = r#"{"name":"contract","desc":"description for contract","networks":{"genesis hash":{"appID":123}},"methods":[{"name":"add","desc":"description","args":[{"name":"0","type":"uint32","desc":"description"},{"name":"1","type":"uint32","desc":"description"}],"returns":{"type":"uint32","desc":"description"}}]}"#;

        let json_res = serde_json::to_string(&contract);
        assert!(json_res.is_ok());
        let json = json_res.unwrap();

        assert_eq!(expected, json);
    }
}
