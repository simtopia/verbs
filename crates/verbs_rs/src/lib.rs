//! # VERBS
//!
//! Ethereum agent based modelling library
//!
//! Verbs is a library designed to aid the
//! implementation of agent based models
//! of the ethereum blockchain with a focus
//! on performance and quick development
//! of simulations.
//!
//! Verbs is built around the
//! [revm](https://github.com/bluealloy/revm)
//! implementation of the Ethereum virtual
//! machine, agents interact directly with
//! EVM, avoiding the overhead of messaging
//! or multi-threading.
//!
//! ## Loading Contracts
//!
//! Verbs makes use of [alloy_sol_types] to convert
//! ABI strings/files to Rust classes that can
//! encode/decode function arguments etc.
//!
//! An ABI rust representation can generated using the `sol!`
//! macro, for example
//!
//! ```
//! use alloy_sol_types::sol;
//!
//! sol!(
//!     ContractName,
//!     r#"[
//!         {
//!             "inputs": [],
//!             "name": "getValue",
//!             "outputs": [
//!                 {
//!                     "internalType": "int256",
//!                     "name": "",
//!                     "type": "int256"
//!                 }
//!             ],
//!             "stateMutability": "view",
//!             "type": "function"
//!         }
//!     ]"#
//! );
//!
//! // Encodes call arguments
//! ContractName::getValueCall {};
//! ```

pub mod agent;
pub mod contract;
mod db;
pub mod env;
pub mod sim_runner;
pub mod utils;

pub use db::{ForkDb, LocalDB, RequestCache, DB};
