use ethabi::token::{LenientTokenizer, Token, Tokenizer};
use ethers_contract::BaseContract;
use ethers_core::abi::{Contract, Tokenize};
use ethers_core::types::Bytes as EthersBytes;
use revm::primitives::{Address, Bytecode, Bytes};

pub struct ContractDefinition {
    pub name: String,
    pub abi: BaseContract,
    pub bytecode: Bytecode,
    pub arguments: Bytes,
    pub deploy_address: Address,
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

pub struct Call {
    pub callee: Address,
    pub transact_to: Address,
    pub args: EthersBytes,
}

impl ContractDefinition {
    pub fn load_abi(abi_path: String) -> BaseContract {
        let abi_file = std::fs::File::open(abi_path).unwrap();
        let abi = Contract::load(abi_file).unwrap();
        BaseContract::from(abi)
    }

    pub fn load_params(
        abi: &BaseContract,
        params_path: String,
        deployment_args: Option<Vec<Token>>,
    ) -> (String, Bytecode, Address, Bytes) {
        let params_file = std::fs::File::open(params_path).unwrap();
        let params_json: serde_json::Value = serde_json::from_reader(params_file).unwrap();

        let name = params_json
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();

        let bytes = params_json.get("bytecode").unwrap().as_str().unwrap();
        let bytes = hex::decode(bytes).expect("Decoding failed");
        let bytes2 = bytes.clone();

        let bytecode = Bytecode::new_raw(Bytes::from(bytes));

        let deploy_address = params_json.get("deploy_address").unwrap().as_str().unwrap();
        let deploy_address = hex::decode(deploy_address).expect("Decoding address failed");
        let deploy_address: Address = Address::from_slice(&deploy_address);

        let constructor_args = params_json
            .get("constructor_args")
            .unwrap()
            .as_array()
            .unwrap();

        let encoded_constructor_args: Bytes;

        if abi.abi().constructor.is_none() {
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
                    let token = LenientTokenizer::tokenize(&b.kind, arg_str).expect(
                        format!("Could not parse token {} as {}", arg_str, b.kind).as_str(),
                    );
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

        (name, bytecode, deploy_address, encoded_constructor_args)
    }

    pub fn load(
        abi_path: String,
        params_path: String,
        deployment_args: Option<Vec<Token>>,
    ) -> ContractDefinition {
        let abi = ContractDefinition::load_abi(abi_path);
        let (name, bytecode, deploy_address, constructor_args) =
            ContractDefinition::load_params(&abi, params_path, deployment_args);

        ContractDefinition {
            name,
            abi,
            bytecode,
            arguments: constructor_args,
            deploy_address,
        }
    }
}
