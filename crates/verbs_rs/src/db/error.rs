use alloy_primitives::{Address, B256, U256};
use ethers_core::types::BlockId;

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum DatabaseError {
    #[error("{0}")]
    Message(String),
    #[error("no cheats available for {0}")]
    NoCheats(Address),
    #[error("failed to fetch AccountInfo {0}")]
    MissingAccount(Address),
    #[error("code should already be loaded: {0}")]
    MissingCode(B256),
    #[error("failed to get account for {0}")]
    GetAccount(Address),
    #[error("failed to get storage for {0} at {1}")]
    GetStorage(Address, U256),
    #[error("failed to get block hash for {0}")]
    GetBlockHash(U256),
    #[error("failed to get full block for {0:?}")]
    GetFullBlock(BlockId),
    #[error("block {0:?} does not exist")]
    BlockNotFound(BlockId),
    #[error("failed to get transaction {0}")]
    GetTransaction(B256),
}

impl DatabaseError {
    /// Create a new error with a message
    pub fn msg(msg: impl Into<String>) -> Self {
        DatabaseError::Message(msg.into())
    }

    /// Create a new error with a message
    pub fn display(msg: impl std::fmt::Display) -> Self {
        DatabaseError::Message(msg.to_string())
    }
}
