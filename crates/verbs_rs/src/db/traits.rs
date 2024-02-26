use super::error::DatabaseError;
use alloy_primitives::Address;
use revm::{
    db::{Database, DbAccount},
    primitives::{AccountInfo, Bytecode, HashMap, Log, B256, U256},
    DatabaseCommit,
};

/// Combined `Revm` database trait with data export methods
///
/// Extends the [Database] and [DatabaseCommit] traits with
/// methods to export the state of the DB. These methods
/// allow the Db state to be exported from the Python API.
pub trait DB: Database<Error = DatabaseError> + DatabaseCommit {
    fn insert_account_info(&mut self, address: Address, account_info: AccountInfo);
    fn accounts(&self) -> &HashMap<Address, DbAccount>;
    fn contracts(&self) -> &HashMap<B256, Bytecode>;
    fn logs(&self) -> &Vec<Log>;
    fn block_hashes(&self) -> &HashMap<U256, B256>;
}
