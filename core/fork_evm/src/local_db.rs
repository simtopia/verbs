use super::db::DB;
use super::error::DatabaseError;
use revm::db::in_memory_db::DbAccount;
use revm::db::{AccountState, DatabaseCommit};
use revm::primitives::{
    hash_map::Entry, Account, AccountInfo, Address, Bytecode, HashMap, Log, B256, KECCAK_EMPTY,
    U256,
};
use revm::Database;

#[derive(Debug, Clone)]
pub struct LocalDB {
    pub accounts: HashMap<Address, DbAccount>,
    pub contracts: HashMap<B256, Bytecode>,
    pub logs: Vec<Log>,
    pub block_hashes: HashMap<U256, B256>,
}

impl Default for LocalDB {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalDB {
    pub fn new() -> Self {
        let mut contracts = HashMap::new();
        contracts.insert(KECCAK_EMPTY, Bytecode::new());
        contracts.insert(B256::ZERO, Bytecode::new());
        Self {
            accounts: HashMap::new(),
            contracts,
            logs: Vec::default(),
            block_hashes: HashMap::new(),
        }
    }

    pub fn insert_contract(&mut self, account: &mut AccountInfo) {
        if let Some(code) = &account.code {
            if !code.is_empty() {
                if account.code_hash == KECCAK_EMPTY {
                    account.code_hash = code.hash_slow();
                }
                self.contracts
                    .entry(account.code_hash)
                    .or_insert_with(|| code.clone());
            }
        }
        if account.code_hash == B256::ZERO {
            account.code_hash = KECCAK_EMPTY;
        }
    }

    pub fn insert_account_info(&mut self, address: Address, mut info: AccountInfo) {
        self.insert_contract(&mut info);
        self.accounts.entry(address).or_default().info = info;
    }

    pub fn load_account(&mut self, address: Address) -> Result<&mut DbAccount, DatabaseError> {
        match self.accounts.entry(address) {
            Entry::Occupied(entry) => Ok(entry.into_mut()),
            Entry::Vacant(_) => Err(DatabaseError::GetAccount(address)),
        }
    }

    pub fn insert_account_storage(
        &mut self,
        address: Address,
        slot: U256,
        value: U256,
    ) -> Result<(), DatabaseError> {
        let account = self.load_account(address)?;
        account.storage.insert(slot, value);
        Ok(())
    }

    pub fn replace_account_storage(
        &mut self,
        address: Address,
        storage: HashMap<U256, U256>,
    ) -> Result<(), DatabaseError> {
        let account = self.load_account(address)?;
        account.account_state = AccountState::StorageCleared;
        account.storage = storage.into_iter().collect();
        Ok(())
    }
}

impl DB for LocalDB {
    fn insert_account_info(&mut self, address: Address, account_info: AccountInfo) {
        self.insert_account_info(address, account_info)
    }

    fn accounts(&self) -> &HashMap<Address, DbAccount> {
        &self.accounts
    }

    fn contracts(&self) -> &HashMap<B256, Bytecode> {
        &self.contracts
    }

    fn logs(&self) -> &Vec<Log> {
        &self.logs
    }

    fn block_hashes(&self) -> &HashMap<U256, B256> {
        &self.block_hashes
    }
}

impl DatabaseCommit for LocalDB {
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        for (address, mut account) in changes {
            if !account.is_touched() {
                continue;
            }
            if account.is_selfdestructed() {
                let db_account = self.accounts.entry(address).or_default();
                db_account.storage.clear();
                db_account.account_state = AccountState::NotExisting;
                db_account.info = AccountInfo::default();
                continue;
            }
            let is_newly_created = account.is_created();
            self.insert_contract(&mut account.info);

            let db_account = self.accounts.entry(address).or_default();
            db_account.info = account.info;

            db_account.account_state = if is_newly_created {
                db_account.storage.clear();
                AccountState::StorageCleared
            } else if db_account.account_state.is_storage_cleared() {
                // Preserve old account state if it already exists
                AccountState::StorageCleared
            } else {
                AccountState::Touched
            };
            db_account.storage.extend(
                account
                    .storage
                    .into_iter()
                    .map(|(key, value)| (key, value.present_value())),
            );
        }
    }
}

impl Database for LocalDB {
    type Error = DatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let basic = match self.accounts.entry(address) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(DbAccount::new_not_existing()),
        };
        Ok(basic.info())
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self.contracts.entry(code_hash) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(_) => Err(DatabaseError::MissingCode(code_hash)),
        }
    }

    /// Get the value in an account's storage slot.
    ///
    /// It is assumed that account is already loaded.
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        match self.accounts.entry(address) {
            Entry::Occupied(mut acc_entry) => {
                let acc_entry = acc_entry.get_mut();
                match acc_entry.storage.entry(index) {
                    Entry::Occupied(entry) => Ok(*entry.get()),
                    Entry::Vacant(_) => {
                        if matches!(
                            acc_entry.account_state,
                            AccountState::StorageCleared | AccountState::NotExisting
                        ) {
                            Ok(U256::ZERO)
                        } else {
                            Err(DatabaseError::GetStorage(address, index))
                        }
                    }
                }
            }
            Entry::Vacant(_) => Err(DatabaseError::GetStorage(address, index)),
        }
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        match self.block_hashes.entry(number) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(_) => Err(DatabaseError::GetBlockHash(number)),
        }
    }
}
