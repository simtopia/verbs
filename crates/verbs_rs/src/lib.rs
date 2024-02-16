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

pub mod agent;
pub mod contract;
pub mod env;
pub mod sim_runner;
pub mod utils;

pub use verbs_db::{ForkDb, LocalDB, RequestCache, DB};
