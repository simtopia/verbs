//! Agent traits and structures
//!
//! Traits defining required agent functionality
//! and some implementations of common data
//! structures of agents for use in simulations.
//!

pub mod agent_vec;
pub mod singleton_agent;
pub mod traits;

pub use agent_vec::*;
pub use singleton_agent::*;
pub use traits::*;
