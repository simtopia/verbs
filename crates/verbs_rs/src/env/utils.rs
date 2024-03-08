//! EVM and data processing utilities
//!

use crate::contract::Event;
use alloy_primitives::{Address, Bytes, U256};
use alloy_sol_types::{decode_revert_reason, SolCall, SolEvent};
use revm::primitives::{ExecutionResult, Log, Output, TransactTo, TxEnv};
use std::fmt;

/// Error raised when an EVM transaction is reverted
#[derive(Debug, Clone)]
pub struct RevertError {
    /// Name of the function that was called
    pub function_name: &'static str,
    /// Address of the sender of the transaction
    sender: Address,
    /// Decoded revert error message
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

/// Process an [ExecutionResult] from a contract deployment
///
/// Process the result of a call to deploy a contract and
/// decode the output or reason message.
///
/// # Arguments
///
/// - `contract_name` - Name of the contract being deployed
/// - `execution_result` - Result returned from the EVM
///
/// # Panics
///
/// Panics if the deployment was reverted or halted.
///
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

/// Initialise a transaction for contract deployment
///
/// Helper function to initialise a transaction to
/// deploy a contract.
///
/// # Arguments
///
/// - `caller` - Address of the account deploying the contract
/// - `data` - ABI encoded deployment bytecode and arguments
///
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

/// Initialise a transaction calling a contract
///
/// Helper function to initialise a transaction
/// calling a contract function.
///
/// # Arguments
///
/// - `caller` - Address of the contract caller
/// - `contract` - Address of the contract to call
/// - `data` - ABI encoded function arguments
/// - `value` - Value attached to the transaction
///
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

/// Handle an [ExecutionResult] returned from a transaction
///
/// # Arguments
///
/// - `sender` - Address of the transaction sender
/// - `execution_result` - [ExecutionResult] returned from a transaction
///
/// # Raises
///
/// Raises a [RevertError] if the transaction is reverted.
///
/// # Panics
///
/// Panics if the transaction is halted.
///
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

/// Process an [ExecutionResult] returning an [Event]
///
/// Creates an [Event] from an [ExecutionResult], where
/// events are stored over the course of the simulation
/// allowing the history of the simulation to be recreated.
///
/// # Arguments
///
/// - `step` - Simulation step
/// - `sequence` - Position in sequence the transaction was
///   executed
/// - `function_selector` - 4 byte function selector of the
///   contract function that was called
/// - `sender` - Address of the transaction sender
/// - `execution_result` - [ExecutionResult] returned from
///   the transaction
/// - `checked` - Flag if `true` a reverted transaction will
///   cause a panic and stop the simulation. Should be set
///   to `false` if it's possible for a transaction to revert
///   but the simulation should continue.
///
/// # Panics
///
/// Panics if the transaction halts and if `checked` is true
/// and the transaction is reverted.
///
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

/// Convert an [ExecutionResult] to an [Output]
///
/// # Arguments
///
/// - `function_name` - Name of the function called
/// - `sender` - Address of the transaction sender
/// - `execution_result` - [ExecutionResult] returned
///   from the transaction
///
/// # Panics
///
/// Panics if the transaction was halted
///
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

/// Decode data attached to an [Event]
///
/// # Arguments
///
/// - `event` - Simulation [Event]
///
/// # Panics
///
/// Panics if the data cannot be decoded for the given event
///
pub fn decode_event<T: SolEvent>(event: &Event) -> (usize, usize, Log<T>) {
    let log = event.logs.last().unwrap();
    let decoded_event = T::decode_log(log, false);

    let decoded_event = match decoded_event {
        Ok(e) => e,
        Err(_) => panic!("Failed to decode event from {:?}", event.function_selector),
    };

    (event.step, event.sequence, decoded_event)
}

/// Filter and process a vector of simulation events
///
/// # Arguments
///
/// - events - Vector of simulation [Event]
///
pub fn process_events<S: SolCall, T: SolEvent>(events: &[Event]) -> Vec<(usize, usize, Log<T>)> {
    let function_selector = S::SELECTOR;
    events
        .iter()
        .filter(|x| x.function_selector == function_selector)
        .map(decode_event)
        .collect()
}
