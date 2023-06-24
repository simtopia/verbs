use crate::contract::structs::ContractDefinition;
use ethabi::token::{LenientTokenizer, Token, Tokenizer};
use ethers_contract::BaseContract;
use ethers_core::abi::Contract;
use revm::primitives::{Address, Bytecode, Bytes, U256};
use serde_json::value::Value as SerdeVale;
use std::collections::HashMap;

pub fn load_abi(abi_path: &str) -> BaseContract {
    let abi_file = std::fs::File::open(abi_path).unwrap();
    let abi = Contract::load(abi_file).unwrap();
    BaseContract::from(abi)
}

fn unpack_storage(v: (&String, &SerdeVale)) -> (U256, U256) {
    let slot = v.0.strip_prefix("0x").unwrap();
    let slot = U256::from_str_radix(slot, 16).unwrap();
    let value = v.1.as_str().unwrap().strip_prefix("0x").unwrap();
    let value = U256::from_str_radix(value, 16).unwrap();

    (slot, value)
}

pub fn load_params(
    abi: &BaseContract,
    params_path: &str,
    deployment_args: Option<Vec<Token>>,
) -> (
    String,
    Bytecode,
    Address,
    Bytes,
    Option<HashMap<U256, U256>>,
) {
    println!("Loading {}", params_path);
    let params_file = std::fs::File::open(params_path).unwrap();
    let params_json: serde_json::Value = serde_json::from_reader(params_file).unwrap();

    let name = params_json
        .get("name")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let bytes = params_json
        .get("bytecode")
        .unwrap()
        .as_str()
        .unwrap()
        .strip_prefix("0x")
        .unwrap();
    let bytes = hex::decode(bytes).expect("Decoding failed");
    let bytes2 = bytes.clone();

    let bytecode = Bytecode::new_raw(Bytes::from(bytes));

    let deploy_address = params_json
        .get("deploy_address")
        .unwrap()
        .as_str()
        .unwrap()
        .strip_prefix("0x")
        .unwrap();
    let deploy_address = hex::decode(deploy_address).expect("Decoding address failed");
    let deploy_address: Address = Address::from_slice(&deploy_address);

    let storage_values = params_json.get("storage");

    let storage_values = match storage_values {
        None => Option::None,
        Some(x) => Option::Some(
            x.as_object()
                .unwrap()
                .into_iter()
                .map(unpack_storage)
                .collect(),
        ),
    };

    let constructor_args = params_json
        .get("constructor_args")
        .unwrap()
        .as_array()
        .unwrap();

    let encoded_constructor_args: Bytes;

    if (abi.abi().constructor.is_none()) || (!storage_values.is_none()) {
        encoded_constructor_args = Bytes::default();
    } else {
        let mut constructor_tokens: Vec<Token>;

        if deployment_args.is_none() {
            constructor_tokens = Vec::new();

            for (a, b) in Iterator::zip(
                constructor_args.iter(),
                abi.abi().constructor().unwrap().clone().inputs,
            ) {
                let arg_str = a.as_str().unwrap();
                let token = LenientTokenizer::tokenize(&b.kind, arg_str)
                    .expect(format!("Could not parse token {} as {}", arg_str, b.kind).as_str());
                constructor_tokens.push(token);
            }
        } else {
            constructor_tokens = deployment_args.unwrap();
        }

        let constructor_args = abi
            .abi()
            .constructor()
            .unwrap()
            .encode_input(bytes2, &constructor_tokens)
            .unwrap();

        encoded_constructor_args = Bytes::from(constructor_args);
    }

    (
        name,
        bytecode,
        deploy_address,
        encoded_constructor_args,
        storage_values,
    )
}

pub fn load_contract(
    abi_path: &str,
    params_path: &str,
    deployment_args: Option<Vec<Token>>,
) -> ContractDefinition {
    let abi = load_abi(abi_path);
    let (name, bytecode, deploy_address, constructor_args, storage_values) =
        load_params(&abi, params_path, deployment_args);

    ContractDefinition {
        name,
        abi,
        bytecode,
        arguments: constructor_args,
        deploy_address,
        storage_values,
    }
}
