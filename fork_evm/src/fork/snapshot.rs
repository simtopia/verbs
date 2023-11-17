use alloy_primitives::{Address, B256, U256};
use revm::{
    primitives::{AccountInfo, Env, HashMap as Map},
    JournaledState,
};
use serde::{Deserialize, Serialize};

/// A minimal abstraction of a state at a certain point in time
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub accounts: Map<Address, AccountInfo>,
    pub storage: Map<Address, Map<U256, U256>>,
    pub block_hashes: Map<U256, B256>,
}

/// Represents a snapshot taken during evm execution
#[derive(Clone, Debug)]
pub struct BackendSnapshot<T> {
    pub db: T,
    /// The journaled_state state at a specific point
    pub journaled_state: JournaledState,
    /// Contains the env at the time of the snapshot
    pub env: Env,
}
