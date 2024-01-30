use alloy_primitives::{Address, U256};
use revm::primitives::{Log, Output};

/// EVM call/transaction
#[derive(Clone)]
pub struct Transaction {
    /// Name of the function being called.
    pub function_selector: [u8; 4],
    /// Address of the contract caller
    pub callee: Address,
    /// Address to transact to
    pub transact_to: Address,
    /// Contract arguments represented as bytes
    pub args: Vec<u8>,
    /// Value attached to the transaction
    pub value: U256,
    /// Flag if `true` the simulation will halt (panic)
    /// if this transaction fails.
    pub checked: bool,
}

// Result if a transaction also wrapping any returned events
pub struct TransactionResult {
    /// Flag whether transaction was successful.
    pub success: bool,
    /// Output data.
    pub output: Output,
    /// Vec of events returned by call.
    pub events: Option<Event>,
}

/// Wraps event logs with additional event information
pub struct Event {
    /// Name of the function being called.
    pub function_selector: [u8; 4],
    /// Event data
    pub logs: Vec<Log>,
    /// Step event was created
    pub step: usize,
    /// Sequence event created inside a block
    pub sequence: usize,
}
