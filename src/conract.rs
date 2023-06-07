use ethabi::token::{LenientTokenizer, Token, Tokenizer};
use ethers_contract::BaseContract;
use ethers_core::abi::Contract;
use ethers_core::abi::Tokenize;
use revm::primitives::{Address, Bytecode, Bytes};

pub struct ContractDefinition {
    pub abi: BaseContract,
    pub bytecode: Bytecode,
    pub arguments: Bytes,
}

pub struct DeployedContract {
    pub abi: BaseContract,
    pub address: Address,
}

pub struct Transaction<T: Tokenize> {
    pub callee: Address,
    pub contract_idx: usize,
    pub function_name: &'static str,
    pub args: T,
}

impl ContractDefinition {
    pub fn load(
        abi_path: String,
        params_path: String,
        deployment_args: Option<Vec<Token>>,
    ) -> ContractDefinition {
        let abi_file = std::fs::File::open(abi_path).unwrap();
        let abi = Contract::load(abi_file).unwrap();
        let abi = BaseContract::from(abi);

        let params_file = std::fs::File::open(params_path).unwrap();
        let params_json: serde_json::Value = serde_json::from_reader(params_file).unwrap();

        let bytes = params_json.get("bin").unwrap().as_str().unwrap();
        let bytes = hex::decode(bytes).expect("Decoding failed");
        let bytes2 = bytes.clone();

        let bytecode = Bytecode::new_raw(Bytes::from(bytes));

        let constructor_args = params_json
            .get("constructor_args")
            .unwrap()
            .as_array()
            .unwrap();

        let mut constructor_tokens: Vec<Token> = Vec::new();

        if deployment_args.is_none() {
            for (a, b) in Iterator::zip(
                constructor_args.iter(),
                abi.as_ref().constructor().unwrap().clone().inputs,
            ) {
                let arg_str = a.as_str().unwrap();
                let token = LenientTokenizer::tokenize(&b.kind, arg_str).unwrap();
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

        let constructor_args = Bytes::from(constructor_args);

        ContractDefinition {
            abi: abi,
            bytecode: bytecode,
            arguments: constructor_args,
        }
    }
}
