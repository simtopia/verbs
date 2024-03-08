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
//! use verbs_rs;
//! use revm::Evm;
//!
//! let db = verbs_rs::LocalDB::new();
//! let mut evm = Evm::builder().with_db(db).build();
//! ````
//!

mod error;
mod fork_db;
mod local_db;
mod provider;
mod runtime_client;
mod traits;
mod types;

pub use fork_db::ForkDb;
pub use local_db::LocalDB;
pub use traits::DB;
pub use types::RequestCache;
