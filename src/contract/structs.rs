use ethers_contract::BaseContract;
use ethers_core::types::Bytes as EthersBytes;
use revm::primitives::{Address, Bytecode, Bytes, Log, Output, U256};
use std::collections::HashMap;

/// Collection of data used to deploy a new contract.
pub struct ContractDefinition {
    /// Name of the contract.
    pub name: String,
    /// Contract ABI object.
    pub abi: BaseContract,
    /// Contract Bytecode.
    pub bytecode: Bytecode,
    /// Constructor arguments packed into bytes.
    pub arguments: Bytes,
    /// Desired deployment address of this contract.
    pub deploy_address: Address,
    /// Map of key-value storage pairs.
    pub storage_values: Option<HashMap<U256, U256>>,
}

/// EVM call/transaction
pub struct Call {
    /// Name of the function being called.
    pub function_name: &'static str,
    /// Index of the contract being called.
    pub contract_idx: usize,
    /// Address of the contract caller
    pub callee: Address,
    /// Address to transact to
    pub transact_to: Address,
    /// Contract arguments represented as bytes
    pub args: EthersBytes,
    /// Flag if `true` the simulation will halt (panic)
    /// if this transaction fails.
    pub checked: bool,
}

// Result if a transaction also wrapping any returned events
pub struct CallResult {
    /// Flag whether transaction was successful.
    pub success: bool,
    /// Output data.
    pub output: Output,
    /// Vec of events returned by call.
    pub events: Vec<Event>,
}

/// Wraps event logs with additional event information
pub struct Event {
    /// Name of the function being called.
    pub function_name: &'static str,
    /// Index of the contract being called.
    pub contract_idx: usize,
    /// Event data
    pub log: Log,
    /// Step event was created
    pub step: i64,
}
