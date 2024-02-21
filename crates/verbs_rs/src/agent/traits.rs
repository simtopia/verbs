//! Traits designating required simulation agent functionality
//!
//! The traits are intended to be used in a hierarchical manner:
//!
//! * [SimState] collect all simulation agents, where fields
//!   may be different agent types. This trait then describes
//!   functions called during simulation execution
//! * [AgentSet] is intended as a homogeneous collection of
//!   an agent type
//! * [Agent] is an individual agent that may be member of an
//!   [AgentSet]
//!
//! Implementers have the flexibility to only use part of this
//! structure though, for instance an implementation of
//! [SimState] could implement an individual agent.
//!
//! Since it is a common use case to want to iterate over a
//! agents of different types, the macro `#[derive(SimState)]`
//! will automatically implement functions that iterate
//! over field containing agents.
//!

use crate::contract::Transaction;
use crate::env::Env;
use crate::DB;
use alloy_primitives::Address;
use fastrand::Rng;
pub use verbs_macros::SimState;

/// Simulation agent state trait
///
/// Trait providing an interface to update the
/// state of all agents over the course of a
/// simulation. This trait can be automatically
/// derived for a struct where the fields
/// are sets of agents of a single type using the
/// [SimState] macro. This will generate the
/// code to automatically iterate
/// over each set of agents in turn.
///
/// # Examples
///
/// ```
/// use fastrand::Rng;
/// use alloy_primitives::Address;
/// use verbs_rs::{DB, env::Env};
/// use verbs_rs::agent::{Agent, RecordedAgent, AgentVec, AgentSet, SimState};
/// use verbs_rs::contract::Transaction;
///
/// struct DummyAgent{}
///
/// impl Agent for DummyAgent {
///     fn update<D: DB>(
///         &mut self, rng: &mut Rng, network: &mut Env<D>
///     ) -> Vec<Transaction> {
///         Vec::default()
///     }
///
///     fn get_address(&self) -> Address {
///         Address::ZERO
///     }
/// }
///
/// impl RecordedAgent<bool> for DummyAgent {
///     fn record<D: DB>(&mut self, _env: &mut Env<D>) -> bool {
///         true
///     }
/// }
///
/// #[derive(SimState)]
/// struct TestState {
///     a: AgentVec::<bool, DummyAgent>,
///     b: AgentVec::<bool, DummyAgent>,
/// }
/// ```
pub trait SimState {
    /// Update the state of all agents, and return any transactions
    fn call_agents<D: DB>(&mut self, rng: &mut Rng, env: &mut Env<D>) -> Vec<Transaction>;
    /// Record the current state of the agents in this set
    fn record_agents<D: DB>(&mut self, env: &mut Env<D>);
}

/// Trait defining behaviour for a single agent
///
/// This is intended to be called for each individual
/// agent at each step of the simulation, updating the
/// state of the agent and recording state data.
/// # Examples
///
/// ```
/// use fastrand::Rng;
/// use alloy_primitives::Address;
/// use verbs_rs::{DB, env::Env};
/// use verbs_rs::agent::{Agent, RecordedAgent, AgentVec, AgentSet};
/// use verbs_rs::contract::Transaction;
///
/// struct DummyAgent{
///     state: i32,
/// }
///
/// impl Agent for DummyAgent {
///     fn update<D: DB>(
///         &mut self, rng: &mut Rng, network: &mut Env<D>
///     ) -> Vec<Transaction> {
///         self.state += 1;
///         Vec::default()
///     }
///
///     fn get_address(&self) -> Address {
///         Address::ZERO
///     }
/// }
/// ```
pub trait Agent {
    /// Update the agent and optionally return a [Transaction]
    /// this should not update the state of the evm directly.
    ///
    /// # Arguments
    ///
    /// * `rng`: Fastrand rng state
    /// * `env`: Simulation environment
    ///
    fn update<D: DB>(&mut self, rng: &mut Rng, env: &mut Env<D>) -> Vec<Transaction>;
    /// Get the address of the agent.
    fn get_address(&self) -> Address;
}

/// Trait used to record the state of the agent over the course of the simulation
///
/// Each step this is called after the state of the simulation
/// is updated, and is intended to record the state of an agent
/// or some part of the state of the EVM. The actual type of data
/// returned is left to the implementer.
///
/// # Examples
///
/// ```
/// use fastrand::Rng;
/// use verbs_rs::{DB, env::Env};
/// use verbs_rs::agent::RecordedAgent;
///
/// struct DummyAgent{
///     current_state: i32
/// }
///
/// impl RecordedAgent<i32> for DummyAgent {
///     fn record<D: DB>(&mut self, _env: &mut Env<D>) -> i32 {
///         self.current_state
///     }
/// }
/// ```
pub trait RecordedAgent<R> {
    /// Get a record of the current state of the agent. Records are
    /// collected as a vector of vectors representing the state of a
    /// collection of agents over the history of the simulation.
    fn record<D: DB>(&mut self, env: &mut Env<D>) -> R;
}

/// A homogenous collection of agents
///
/// Designed to represent a group of agents of a uniform type
/// and update and record the group state at each step of the
/// simulation.
pub trait AgentSet {
    /// Update all the agents in the set, collecting any EVM calls generated by the agents
    ///
    /// # Arguments
    ///
    /// * `rng` - Fastrand rng state
    /// * `env` - Simulation environment
    ///
    fn call<D: DB>(&mut self, rng: &mut fastrand::Rng, env: &mut Env<D>) -> Vec<Transaction>;
    /// Record the state of all the agents
    fn record<D: DB>(&mut self, env: &mut Env<D>);
    /// Get a vector of agent addresses contained in this set
    fn get_addresses(&self) -> Vec<Address>;
}

/// Take ownership of time-series data from a set of agents
///
/// Returns a time series of vectors of records across
/// all the agents in the set.
pub trait RecordedAgentSet<R> {
    fn take_records(&mut self) -> Vec<Vec<R>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LocalDB;
    use alloy_primitives::{Address, U256};

    struct DummyAgentSet {
        v: bool,
    }

    impl AgentSet for DummyAgentSet {
        fn call<D: DB>(&mut self, _rng: &mut Rng, _env: &mut Env<D>) -> Vec<Transaction> {
            vec![Transaction {
                function_selector: [0, 0, 0, 0],
                callee: Address::ZERO,
                transact_to: Address::ZERO,
                args: Vec::default(),
                value: U256::ZERO,
                checked: self.v,
            }]
        }

        fn record<D: DB>(&mut self, _env: &mut Env<D>) {}

        fn get_addresses(&self) -> Vec<Address> {
            vec![Address::ZERO]
        }
    }

    #[test]
    fn test_macro() {
        #[derive(SimState)]
        struct TestState {
            a: DummyAgentSet,
            b: DummyAgentSet,
        }

        let mut x = TestState {
            a: DummyAgentSet { v: true },
            b: DummyAgentSet { v: false },
        };

        let mut rng = fastrand::Rng::with_seed(101);
        let mut network = &mut Env::<LocalDB>::init(U256::ZERO, U256::ZERO);

        let calls = x.call_agents(&mut rng, &mut network);

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].checked, true);
        assert_eq!(calls[1].checked, false);
    }
}
