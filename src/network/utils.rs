use crate::contract::{Call, CallResult, Event};
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{decode_revert_reason, SolCall, SolEvent};
use log::warn;
use revm::primitives::{ExecutionResult, Log, Output, TransactTo, TxEnv};
use std::fmt;

#[derive(Debug, Clone)]
pub struct RevertError {
    pub function_name: &'static str,
    output: Option<String>,
}

impl fmt::Display for RevertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out_str = match &self.output {
            Some(x) => x.as_str(),
            None => "No output",
        };

        write!(
            f,
            "Failed to call {} due to revert: {}",
            self.function_name, out_str,
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

pub fn result_to_output_with_events(
    step: usize,
    sequence: usize,
    function_name: &'static str,
    execution_result: ExecutionResult,
    checked: bool,
) -> CallResult {
    match execution_result {
        ExecutionResult::Success { output, logs, .. } => match output {
            Output::Call(_) => CallResult {
                success: true,
                output,
                events: Some(Event {
                    function_name,
                    logs,
                    step,
                    sequence,
                }),
            },
            Output::Create(..) => {
                panic!("Unexpected call to create contract during simulation.")
            }
        },
        ExecutionResult::Revert { output, .. } => {
            if checked {
                panic!(
                    "Failed to call {} due to revert: {:?}",
                    function_name,
                    decode_revert_reason(&output.0)
                )
            } else {
                warn!(
                    "Failed to call {} due to revert: {:?}",
                    function_name,
                    decode_revert_reason(&output.0)
                );
                CallResult {
                    success: false,
                    output: Output::Call(Bytes::default()),
                    events: None,
                }
            }
        }
        ExecutionResult::Halt { reason, .. } => {
            panic!("Failed to call {} due to halt: {:?}", function_name, reason)
        }
    }
}

pub fn result_to_output(
    function_name: &'static str,
    execution_result: ExecutionResult,
) -> Result<(Output, Vec<Log>), RevertError> {
    match execution_result {
        ExecutionResult::Success { output, logs, .. } => Ok((output, logs)),
        ExecutionResult::Revert { output, .. } => Err(RevertError {
            function_name,
            output: decode_revert_reason(&output),
        }),
        ExecutionResult::Halt { reason, .. } => {
            panic!("Failed to call {} due to halt: {:?}", function_name, reason)
        }
    }
}

pub fn create_call<T: SolCall>(callee: Address, contract: Address, args: T, checked: bool) -> Call {
    Call {
        function_name: T::SIGNATURE,
        callee,
        transact_to: contract,
        args: args.abi_encode(),
        checked,
    }
}

fn decode_event<T: SolEvent>(event: &Event) -> (usize, usize, T) {
    let log = event.logs.last().unwrap();
    let decoded_event = T::decode_log(log.topics.clone(), log.data.as_ref(), false).unwrap();

    (event.step, event.sequence, decoded_event)
}

pub fn process_events<S: SolCall, T: SolEvent>(events: &Vec<Event>) -> Vec<(usize, usize, T)> {
    let function_name = S::SIGNATURE;
    events
        .into_iter()
        .filter(|x| x.function_name == function_name)
        .map(decode_event)
        .collect()
}
