//! A revm database that forks off a remote client

use super::error::DatabaseError;
use super::snapshot::StateSnapshot;
use crate::fork::{BlockchainDb, SharedBackend};
use alloy_primitives::{Address, B256, U256};
use ethers_core::types::BlockId;
use revm::{
    db::{CacheDB, DatabaseRef},
    primitives::{Account, AccountInfo, Bytecode, HashMap as Map},
    Database, DatabaseCommit,
};

/// a [revm::Database] that's forked off another client
///
/// The `backend` is used to retrieve (missing) data, which is then fetched from the remote
/// endpoint. The inner in-memory database holds this storage and will be used for write operations.
/// This database uses the `backend` for read and the `db` for write operations. But note the
/// `backend` will also write (missing) data to the `db` in the background
#[derive(Debug, Clone)]
pub struct ForkedDatabase {
    /// responsible for fetching missing data
    ///
    /// This is responsible for getting data
    backend: SharedBackend,
    /// Cached Database layer, ensures that changes are not written to the database that
    /// exclusively stores the state of the remote client.
    ///
    /// This separates Read/Write operations
    ///   - reads from the `SharedBackend as DatabaseRef` writes to the internal cache storage
    cache_db: CacheDB<SharedBackend>,
    /// Contains all the data already fetched
    ///
    /// This exclusively stores the _unchanged_ remote client state
    db: BlockchainDb,
    // /// holds the snapshot state of a blockchain
    // snapshots: Arc<Mutex<Snapshots<ForkDbSnapshot>>>,
}

impl ForkedDatabase {
    /// Creates a new instance of this DB
    pub fn new(backend: SharedBackend, db: BlockchainDb) -> Self {
        Self {
            cache_db: CacheDB::new(backend.clone()),
            backend,
            db,
            // snapshots: Arc::new(Mutex::new(Default::default())),
        }
    }

    pub fn database(&self) -> &CacheDB<SharedBackend> {
        &self.cache_db
    }

    pub fn database_mut(&mut self) -> &mut CacheDB<SharedBackend> {
        &mut self.cache_db
    }

    /// Reset the fork to a fresh forked state, and optionally update the fork config
    pub fn reset(
        &mut self,
        _url: Option<String>,
        block_number: impl Into<BlockId>,
    ) -> Result<(), String> {
        self.backend
            .set_pinned_block(block_number)
            .map_err(|err| err.to_string())?;

        // TODO need to find a way to update generic provider via url

        // wipe the storage retrieved from remote
        self.inner().db().clear();
        // create a fresh `CacheDB`, effectively wiping modified state
        self.cache_db = CacheDB::new(self.backend.clone());
        tracing::trace!(target: "backend::forkdb", "Cleared database");
        Ok(())
    }

    /// Flushes the cache to disk if configured
    pub fn flush_cache(&self) {
        self.db.cache().flush()
    }

    /// Returns the database that holds the remote state
    pub fn inner(&self) -> &BlockchainDb {
        &self.db
    }

    pub fn create_snapshot(&self) -> ForkDbSnapshot {
        let db = self.db.db();
        let snapshot = StateSnapshot {
            accounts: db.accounts.read().clone(),
            storage: db.storage.read().clone(),
            block_hashes: db.block_hashes.read().clone(),
        };
        ForkDbSnapshot {
            local: self.cache_db.clone(),
            snapshot,
        }
    }
}

impl Database for ForkedDatabase {
    type Error = DatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        // Note: this will always return Some, since the `SharedBackend` will always load the
        // account, this differs from `<CacheDB as Database>::basic`, See also
        // [MemDb::ensure_loaded](crate::backend::MemDb::ensure_loaded)
        Database::basic(&mut self.cache_db, address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        Database::code_by_hash(&mut self.cache_db, code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        Database::storage(&mut self.cache_db, address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        Database::block_hash(&mut self.cache_db, number)
    }
}

impl DatabaseRef for ForkedDatabase {
    type Error = DatabaseError;

    fn basic(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.cache_db.basic(address)
    }

    fn code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.cache_db.code_by_hash(code_hash)
    }

    fn storage(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        DatabaseRef::storage(&self.cache_db, address, index)
    }

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        self.cache_db.block_hash(number)
    }
}

impl DatabaseCommit for ForkedDatabase {
    fn commit(&mut self, changes: Map<Address, Account>) {
        self.database_mut().commit(changes)
    }
}

/// Represents a snapshot of the database
///
/// This mimics `revm::CacheDB`
#[derive(Clone, Debug)]
pub struct ForkDbSnapshot {
    pub local: CacheDB<SharedBackend>,
    pub snapshot: StateSnapshot,
}

// === impl DbSnapshot ===

impl ForkDbSnapshot {
    fn get_storage(&self, address: Address, index: U256) -> Option<U256> {
        self.local
            .accounts
            .get(&address)
            .and_then(|account| account.storage.get(&index))
            .copied()
    }
}

// This `DatabaseRef` implementation works similar to `CacheDB` which prioritizes modified elements,
// and uses another db as fallback
// We prioritize stored changed accounts/storage
impl DatabaseRef for ForkDbSnapshot {
    type Error = DatabaseError;

    fn basic(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        match self.local.accounts.get(&address) {
            Some(account) => Ok(Some(account.info.clone())),
            None => {
                let mut acc = self.snapshot.accounts.get(&address).cloned();

                if acc.is_none() {
                    acc = self.local.basic(address)?;
                }
                Ok(acc)
            }
        }
    }

    fn code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.local.code_by_hash(code_hash)
    }

    fn storage(&self, address: Address, index: U256) -> Result<U256, Self::Error> {
        match self.local.accounts.get(&address) {
            Some(account) => match account.storage.get(&index) {
                Some(entry) => Ok(*entry),
                None => match self.get_storage(address, index) {
                    None => DatabaseRef::storage(&self.local, address, index),
                    Some(storage) => Ok(storage),
                },
            },
            None => match self.get_storage(address, index) {
                None => DatabaseRef::storage(&self.local, address, index),
                Some(storage) => Ok(storage),
            },
        }
    }

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        match self.snapshot.block_hashes.get(&number).copied() {
            None => self.local.block_hash(number),
            Some(block_hash) => Ok(block_hash),
        }
    }
}
