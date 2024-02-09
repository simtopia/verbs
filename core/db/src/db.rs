use alloy_primitives::Address;
use revm::{
    db::{Database, DbAccount},
    primitives::{AccountInfo, Bytecode, HashMap, Log, B256, U256},
    DatabaseCommit,
};

pub trait DB: Database + DatabaseCommit {
    fn insert_account_info(&mut self, address: Address, account_info: AccountInfo);
    fn accounts(&self) -> &HashMap<Address, DbAccount>;
    fn contracts(&self) -> &HashMap<B256, Bytecode>;
    fn logs(&self) -> &Vec<Log>;
    fn block_hashes(&self) -> &HashMap<U256, B256>;
}
