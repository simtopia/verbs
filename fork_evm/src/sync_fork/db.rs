use alloy_primitives::{Address, B256, U256};
use revm::primitives::{AccountInfo, HashMap as Map};

pub type StorageInfo = Map<U256, U256>;

#[derive(Debug, Default)]
pub struct MemDb {
    /// Account related data
    pub accounts: Map<Address, AccountInfo>,
    /// Storage related data
    pub storage: Map<Address, StorageInfo>,
    /// All retrieved block hashes
    pub block_hashes: Map<U256, B256>,
}

impl MemDb {
    pub fn accounts(&mut self) -> &mut Map<Address, AccountInfo> {
        &mut self.accounts
    }

    pub fn storage(&mut self) -> &mut Map<Address, StorageInfo> {
        &mut self.storage
    }

    pub fn block_hashes(&mut self) -> &mut Map<U256, B256> {
        &mut self.block_hashes
    }
}
