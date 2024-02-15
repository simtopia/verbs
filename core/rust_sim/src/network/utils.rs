use crate::contract::{Event, Transaction};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{decode_revert_reason, SolCall, SolEvent};
use revm::primitives::{ExecutionResult, Log, Output, TransactTo, TxEnv};
use std::fmt;

#[derive(Debug, Clone)]
pub struct RevertError {
    pub function_name: &'static str,
    sender: Address,
    pub output: Option<String>,
}

impl fmt::Display for RevertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out_str = match &self.output {
            Some(x) => x.as_str(),
            None => "No output",
        };

        write!(
            f,
            "Failed to call {} from {} due to revert: {}",
            self.function_name, self.sender, out_str,
        )
    }
}

pub fn deployment_output(contract_name: &str, execution_result: ExecutionResult) -> Output {
    match execution_result {
        ExecutionResult::Success { output, .. } => output,
        ExecutionResult::Revert { output, .. } => {
            panic!(
                "Failed to deploy {} due to revert: {:?}",
                contract_name,
                decode_revert_reason(&output.0)
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

pub fn init_call_transaction(
    caller: Address,
    contract: Address,
    data: Vec<u8>,
    value: U256,
) -> TxEnv {
    TxEnv {
        caller,
        gas_limit: u64::MAX,
        gas_price: U256::ZERO,
        gas_priority_fee: None,
        transact_to: TransactTo::Call(contract),
        value,
        data: Bytes::from(data),
        chain_id: None,
        nonce: None,
        access_list: Vec::new(),
        blob_hashes: Vec::default(),
        max_fee_per_blob_gas: None,
    }
}

pub fn result_to_raw_output(
    sender: Address,
    execution_result: ExecutionResult,
) -> Result<ExecutionResult, RevertError> {
    match execution_result {
        ExecutionResult::Success { .. } => Ok(execution_result),
        ExecutionResult::Revert { output, .. } => Err(RevertError {
            function_name: "Direct execute raw",
            sender,
            output: decode_revert_reason(&output),
        }),
        ExecutionResult::Halt { reason, .. } => panic!("Failed due to halt: {:?}", reason),
    }
}

pub fn result_to_output_with_events(
    step: usize,
    sequence: usize,
    function_selector: [u8; 4],
    sender: Address,
    execution_result: ExecutionResult,
    checked: bool,
) -> Event {
    match execution_result {
        ExecutionResult::Success { output, logs, .. } => match output {
            Output::Call(_) => Event {
                success: true,
                function_selector,
                logs,
                step,
                sequence,
            },
            Output::Create(..) => {
                panic!("Unexpected call to create contract during simulation.")
            }
        },
        ExecutionResult::Revert { output, .. } => match checked {
            true => panic!(
                "Failed to call {:?} from {} due to revert: {:?}",
                function_selector,
                sender,
                decode_revert_reason(&output.0)
            ),
            false => Event {
                success: true,
                function_selector,
                logs: Vec::default(),
                step,
                sequence,
            },
        },
        ExecutionResult::Halt { reason, .. } => {
            panic!(
                "Failed to call {:?} from {} due to halt: {:?}",
                function_selector, sender, reason
            )
        }
    }
}

pub fn result_to_output(
    function_name: &'static str,
    sender: Address,
    execution_result: ExecutionResult,
) -> Result<(Output, Vec<Log>), RevertError> {
    match execution_result {
        ExecutionResult::Success { output, logs, .. } => Ok((output, logs)),
        ExecutionResult::Revert { output, .. } => Err(RevertError {
            function_name,
            sender,
            output: decode_revert_reason(&output),
        }),
        ExecutionResult::Halt { reason, .. } => {
            panic!(
                "Failed to call {} from {} due to halt: {:?}",
                function_name, sender, reason
            )
        }
    }
}

pub fn create_call<T: SolCall>(
    callee: Address,
    contract: Address,
    args: T,
    value: U256,
    checked: bool,
) -> Transaction {
    Transaction {
        function_selector: T::SELECTOR,
        callee,
        transact_to: contract,
        args: args.abi_encode(),
        value,
        checked,
    }
}

pub fn decode_event<T: SolEvent>(event: &Event) -> (usize, usize, T) {
    let log = event.logs.last().unwrap();
    let decoded_event = T::decode_log(log.topics.clone(), log.data.as_ref(), false);

    let decoded_event = match decoded_event {
        Ok(e) => e,
        Err(_) => panic!("Failed to decode event from {:?}", event.function_selector),
    };

    (event.step, event.sequence, decoded_event)
}

pub fn process_events<S: SolCall, T: SolEvent>(events: &[Event]) -> Vec<(usize, usize, T)> {
    let function_selector = S::SELECTOR;
    events
        .iter()
        .filter(|x| x.function_selector == function_selector)
        .map(decode_event)
        .collect()
}
