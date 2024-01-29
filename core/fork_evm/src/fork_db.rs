use super::db::DB;
use super::error::DatabaseError;
use super::provider::ProviderBuilder;
use crate::runtime_client::RuntimeClient;
use crate::types::{ToAlloy, ToEthers};
use alloy_primitives::{keccak256, Bytes};
pub use ethers_core::types::BlockNumber;
use ethers_core::types::{BigEndianHash, Block, BlockId, NameOrAddress, H256};
use ethers_providers::{Middleware, Provider, ProviderError};
use eyre::anyhow;
use revm::db::in_memory_db::DbAccount;
use revm::db::{AccountState, DatabaseCommit};
use revm::primitives::{
    hash_map::Entry, Account, AccountInfo, Address, Bytecode, HashMap, Log, B256, KECCAK_EMPTY,
    U256,
};
use revm::Database;

#[derive(Debug, Clone)]
pub struct ForkDb {
    pub accounts: HashMap<Address, DbAccount>,
    pub contracts: HashMap<B256, Bytecode>,
    pub logs: Vec<Log>,
    pub block_hashes: HashMap<U256, B256>,
    provider: Provider<RuntimeClient>,
    block_id: Option<BlockId>,
    pub block: Block<H256>,
}

impl ForkDb {
    pub fn new(node_url: &str, block_number: u64) -> Self {
        let block_number = match block_number {
            0 => BlockNumber::Latest,
            n => BlockNumber::Number(n.into()),
        };

        let provider = ProviderBuilder::new(node_url).build().unwrap();

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let block = rt
            .block_on(provider.get_block(block_number))
            .unwrap()
            .ok_or(anyhow!("failed to retrieve block"))
            .unwrap();

        let mut contracts = HashMap::new();
        contracts.insert(KECCAK_EMPTY, Bytecode::new());
        contracts.insert(B256::ZERO, Bytecode::new());
        Self {
            accounts: HashMap::new(),
            contracts,
            logs: Vec::default(),
            block_hashes: HashMap::new(),
            provider,
            block_id: Some(block.number.unwrap().into()),
            block,
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

impl DB for ForkDb {
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

impl DatabaseCommit for ForkDb {
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

impl Database for ForkDb {
    type Error = DatabaseError;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        let basic = self.accounts.entry(address);
        let basic = match basic {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let info = basic_from_fork(&self.provider, address, self.block_id);
                let account = match info {
                    Ok(i) => DbAccount {
                        info: i,
                        ..Default::default()
                    },
                    Err(_) => DbAccount::new_not_existing(),
                };
                entry.insert(account)
            }
        };
        Ok(basic.info())
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        match self.contracts.entry(code_hash) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(_) => Err(DatabaseError::MissingCode(code_hash)),
        }
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        match self.accounts.entry(address) {
            Entry::Occupied(mut acc_entry) => {
                let acc_entry = acc_entry.get_mut();
                match acc_entry.storage.entry(index) {
                    Entry::Occupied(entry) => Ok(*entry.get()),
                    Entry::Vacant(entry) => {
                        if matches!(
                            acc_entry.account_state,
                            AccountState::StorageCleared | AccountState::NotExisting
                        ) {
                            Ok(U256::ZERO)
                        } else {
                            let slot =
                                storage_from_fork(&self.provider, address, index, self.block_id);
                            match slot {
                                Ok(s) => {
                                    entry.insert(s);
                                    Ok(s)
                                }
                                Err(_) => Err(DatabaseError::GetStorage(address, index)),
                            }
                        }
                    }
                }
            }
            Entry::Vacant(_) => Err(DatabaseError::GetAccount(address)),
        }
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        match self.block_hashes.entry(number) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(entry) => {
                let hash = block_hash_from_fork(&self.provider, number);
                match hash {
                    Ok(h) => {
                        entry.insert(h);
                        Ok(h)
                    }
                    Err(_) => Err(DatabaseError::GetBlockHash(number)),
                }
            }
        }
    }
}

fn basic_from_fork(
    provider: &Provider<RuntimeClient>,
    address: Address,
    block_id: Option<BlockId>,
) -> Result<AccountInfo, ProviderError> {
    let add = NameOrAddress::Address(address.to_ethers());

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let balance = rt.block_on(provider.get_balance(add.clone(), block_id))?;
    let nonce = rt.block_on(provider.get_transaction_count(add.clone(), block_id))?;
    let code = rt.block_on(provider.get_code(add, block_id))?;
    let code = Bytes::from(code.0);

    let (code, code_hash) = if !code.is_empty() {
        (code.clone(), keccak256(&code))
    } else {
        (Bytes::default(), KECCAK_EMPTY)
    };

    Ok(AccountInfo {
        balance: balance.to_alloy(),
        nonce: nonce.as_u64(),
        code_hash,
        code: Some(Bytecode::new_raw(code).to_checked()),
    })
}

fn storage_from_fork(
    provider: &Provider<RuntimeClient>,
    address: Address,
    index: U256,
    block_id: Option<BlockId>,
) -> Result<U256, ProviderError> {
    let idx_req = B256::from(index);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let storage = rt.block_on(provider.get_storage_at(
        NameOrAddress::Address(address.to_ethers()),
        idx_req.to_ethers(),
        block_id,
    ))?;
    Ok(storage.into_uint().to_alloy())
}

fn block_hash_from_fork(
    provider: &Provider<RuntimeClient>,
    number: U256,
) -> Result<B256, ProviderError> {
    let n: u64 = number.try_into().unwrap();
    let block_id = BlockId::from(n);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let block = rt.block_on(provider.get_block(block_id));

    match block {
        Ok(Some(block)) => Ok(block
            .hash
            .expect("empty block hash on mined block, this should never happen")
            .to_alloy()),
        Ok(None) => Ok(KECCAK_EMPTY),
        Err(e) => Err(e),
    }
}
