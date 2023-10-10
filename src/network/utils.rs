use alloy_primitives::{Address, Bytes, U256};
use revm::primitives::{ExecutionResult, Output, TransactTo, TxEnv};

pub fn deployment_output(contract_name: &str, execution_result: ExecutionResult) -> Output {
    match execution_result {
        ExecutionResult::Success { output, .. } => output,
        ExecutionResult::Revert { output, .. } => {
            panic!(
                "Failed to deploy {} due to revert: {:?}",
                contract_name, output
            )
        }
        ExecutionResult::Halt { reason, .. } => {
            panic!(
                "Failed to deploy {} due to halt: {:?}",
                contract_name, reason
            )
        }
    }
}

pub fn init_create_transaction(caller: Address, data: Vec<u8>) -> TxEnv {
    TxEnv {
        caller,
        gas_limit: u64::MAX,
        gas_price: U256::ZERO,
        gas_priority_fee: None,
        transact_to: TransactTo::create(),
        value: U256::ZERO,
        data: Bytes::from(data),
        chain_id: None,
        nonce: None,
        access_list: Vec::new(),
        blob_hashes: Vec::default(),
        max_fee_per_blob_gas: None,
    }
}

pub fn init_create_call_transaction(caller: Address, contract: Address, data: Vec<u8>) -> TxEnv {
    TxEnv {
        caller,
        gas_limit: u64::MAX,
        gas_price: U256::ZERO,
        gas_priority_fee: None,
        transact_to: TransactTo::Call(contract),
        value: U256::ZERO,
        data: Bytes::from(data),
        chain_id: None,
        nonce: None,
        access_list: Vec::new(),
        blob_hashes: Vec::default(),
        max_fee_per_blob_gas: None,
    }
}
