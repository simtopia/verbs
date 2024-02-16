use crate::contract::Transaction;
use crate::network::Network;
use alloy_primitives::Address;
use db::DB;
use fastrand::Rng;
pub use sim_macros::SimState;

pub trait SimState {
    fn call_agents<D: DB>(&mut self, rng: &mut Rng, network: &mut Network<D>) -> Vec<Transaction>;
    fn record_agents(&mut self);
}

/// Agent trait used to update all agents each model update.
pub trait Agent {
    /// Update the agent and optionally return a call to the EVM
    /// this should not update the state of the evm directly.
    ///
    /// # Arguments
    ///
    /// * `rng` - Fastrand rng state
    /// * `network` - Protocol deployment(s)
    ///
    fn update<D: DB>(&mut self, rng: &mut Rng, network: &mut Network<D>) -> Vec<Transaction>;
    /// Get the address of the agent.
    fn get_address(&self) -> Address;
}

/// Trait used to record the state of the agent over the course of the simulation
pub trait RecordedAgent<R> {
    /// Get a record of the current state of the agent. Records are
    /// collected as a vector of vectors representing the state of a
    /// collection of agents over the history of the simulation.
    fn record(&mut self) -> R;
}

/// A collection of homogenous agents
pub trait AgentSet {
    /// Update all the agents in the set, collecting any EVM calls generated by the agents
    ///
    /// # Arguments
    ///
    /// * `rng` - Fastrand rng state
    /// * `network` - Protocol deployment(s)
    ///
    fn call<D: DB>(
        &mut self,
        rng: &mut fastrand::Rng,
        network: &mut Network<D>,
    ) -> Vec<Transaction>;
    /// Record the state of all the agents
    fn record(&mut self);
    /// Get a vector of agent addresses contained in this set
    fn get_addresses(&self) -> Vec<Address>;
}

// Take time-series data from a set of agents
pub trait RecordedAgentSet<R> {
    fn take_records(&mut self) -> Vec<Vec<R>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{Address, U256};
    use db::LocalDB;

    struct DummyAgentSet {
        v: bool,
    }

    impl AgentSet for DummyAgentSet {
        fn call<D: DB>(&mut self, _rng: &mut Rng, _network: &mut Network<D>) -> Vec<Transaction> {
            vec![Transaction {
                function_selector: [0, 0, 0, 0],
                callee: Address::ZERO,
                transact_to: Address::ZERO,
                args: Vec::default(),
                value: U256::ZERO,
                checked: self.v,
            }]
        }

        fn record(&mut self) {}

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
        let mut network = &mut Network::<LocalDB>::init(U256::ZERO, U256::ZERO);

        let calls = x.call_agents(&mut rng, &mut network);

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].checked, true);
        assert_eq!(calls[1].checked, false);
    }
}
