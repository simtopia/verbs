//! A streamlined single-threaded database for revm
//!
//! Implementations of revm database traits
//! supporting both local and forked
//! functionality. This is intended for use
//! as part of a simulated blockchain and so
//! removes any multi-threaded access or
//! secondary cache databases.
//!
//! # Examples
//!
//! ````
//! use verbs_db;
//! use revm::EVM;
//!
//! let db = verbs_db::LocalDB::new();
//! let mut evm = EVM::new();
//! evm.database(db);
//! ````
//!

mod db;
mod error;
mod fork_db;
mod local_db;
mod provider;
mod runtime_client;
mod types;

pub use db::DB;
pub use fork_db::ForkDb;
pub use local_db::LocalDB;
pub use types::RequestCache;
