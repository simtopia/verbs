use ethers_contract::BaseContract;
use ethers_core::abi::Tokenize;
use ethers_core::types::Bytes as EthersBytes;
use revm::primitives::{Address, Bytecode, Bytes, U256};
use std::collections::HashMap;

pub struct ContractDefinition {
    pub name: String,
    pub abi: BaseContract,
    pub bytecode: Bytecode,
    pub arguments: Bytes,
    pub deploy_address: Address,
    pub storage_values: HashMap<U256, U256>,
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
