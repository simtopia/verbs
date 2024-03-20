//! Vector data structure of simulation agents
//!
//! Data structure that stores a vector of agents
//! of one type. Implements functionality to
//! iterate over and update agents, and
//! record and retrieve simulated agent data.
//!

use crate::agent::traits::{Agent, AgentSet, RecordedAgent, RecordedAgentSet};
use crate::contract::Transaction;
use crate::env::{Env, Validator};
use crate::DB;
use alloy_primitives::Address;
use rand::RngCore;
use std::mem;

/// Implementation of agent set storing agents as a vector
///
/// Stores a vector of agents of a single type,
/// and stores records of their state.
///
/// # Examples
///
/// ```
/// use rand::RngCore;
/// use alloy_primitives::Address;
/// use verbs_rs::{DB, env::{Env, Validator}};
/// use verbs_rs::agent::{Agent, RecordedAgent, AgentVec, AgentSet};
/// use verbs_rs::contract::Transaction;
///
/// struct DummyAgent{}
///
/// impl Agent for DummyAgent {
///     fn update<D: DB, V: Validator, R: RngCore>(
///         &mut self, rng: &mut R, network: &mut Env<D, V>
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
///     fn record<D: DB, V: Validator>(&mut self, _env: &mut Env<D, V>) -> bool {
///         true
///     }
/// }
///
/// let agent_vec = AgentVec::<bool, DummyAgent>::from(
///     vec![DummyAgent{}, DummyAgent{}]
/// );
///
/// let addresses = agent_vec.get_addresses();
/// ```
pub struct AgentVec<R, A: Agent + RecordedAgent<R>> {
    /// Vector of agents of a single type
    agents: Vec<A>,
    /// Records of agent states over the course of the simulation
    records: Vec<Vec<R>>,
}

impl<R, A: Agent + RecordedAgent<R>> Default for AgentVec<R, A> {
    fn default() -> Self {
        AgentVec {
            agents: Vec::<A>::new(),
            records: Vec::<Vec<R>>::new(),
        }
    }
}

impl<R, A: Agent + RecordedAgent<R>> AgentVec<R, A> {
    /// Initialise an empty vector agent-set
    pub fn new() -> Self {
        AgentVec {
            agents: Vec::<A>::new(),
            records: Vec::<Vec<R>>::new(),
        }
    }
    /// Initialise an agent-vec from an existing vector of agents
    ///
    /// # Arguments
    ///
    /// * `agents` - Vector af agents of this type
    ///
    pub fn from(agents: Vec<A>) -> Self {
        AgentVec {
            agents,
            records: Vec::<Vec<R>>::new(),
        }
    }
    /// Insert an agent into the set.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agents of this type
    ///
    pub fn add_agent(&mut self, agent: A) {
        self.agents.push(agent);
    }
    /// Get the recorded history of agents in this set.
    pub fn get_records(&self) -> &Vec<Vec<R>> {
        &self.records
    }
}

impl<R, A: Agent + RecordedAgent<R>> RecordedAgentSet<R> for AgentVec<R, A> {
    /// Take the vector of agent records from the set
    fn take_records(&mut self) -> Vec<Vec<R>> {
        mem::take(&mut self.records)
    }
}

/// Implementations of agent updates and recording.
impl<R: 'static, A: Agent + RecordedAgent<R> + 'static> AgentSet for AgentVec<R, A> {
    /// Call the agents in the set and collect any returned EVM transactions
    ///
    /// This is called during the simulation, updating the state of
    /// the agents, and collecting any submitted transactions into
    /// a single vector.
    ///
    /// # Arguments
    ///
    /// * `rng` - Random generator
    /// * `network` - Protocol deployment(s)
    ///
    fn call<D: DB, V: Validator, RG: RngCore>(
        &mut self,
        rng: &mut RG,
        network: &mut Env<D, V>,
    ) -> Vec<Transaction> {
        self.agents
            .iter_mut()
            .flat_map(|x| x.update(rng, network))
            .collect()
    }
    /// Record the current state of the agents in this set
    fn record<D: DB, V: Validator>(&mut self, env: &mut Env<D, V>) {
        let records: Vec<R> = self.agents.iter_mut().map(|x| x.record(env)).collect();
        self.records.push(records);
    }
    /// Get the addresses of the agents in this set.
    fn get_addresses(&self) -> Vec<Address> {
        self.agents.iter().map(|x| x.get_address()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LocalDB;
    use crate::{agent::traits, env::RandomValidator};
    use alloy_primitives::{Uint, U256};
    use rand::SeedableRng;
    use rstest::*;

    struct TestAgent {
        address: Address,
        value: u64,
    }

    impl traits::Agent for TestAgent {
        fn update<D: DB, V: Validator, RG: RngCore>(
            &mut self,
            _rng: &mut RG,
            _network: &mut crate::env::Env<D, V>,
        ) -> Vec<crate::contract::Transaction> {
            self.value += 1;
            vec![
                Transaction {
                    function_selector: [0, 0, 0, 0],
                    callee: Address::ZERO,
                    transact_to: Address::ZERO,
                    args: Vec::default(),
                    value: U256::ZERO,
                    checked: false,
                    gas_priority_fee: None,
                    nonce: None,
                },
                Transaction {
                    function_selector: [0, 0, 0, 0],
                    callee: Address::ZERO,
                    transact_to: Address::ZERO,
                    args: Vec::default(),
                    value: U256::ZERO,
                    checked: false,
                    gas_priority_fee: None,
                    nonce: None,
                },
            ]
        }

        fn get_address(&self) -> Address {
            self.address
        }
    }

    impl traits::RecordedAgent<u64> for TestAgent {
        fn record<D: DB, V: Validator>(&mut self, _env: &mut Env<D, V>) -> u64 {
            self.value
        }
    }

    #[fixture]
    fn env() -> Env<LocalDB, RandomValidator> {
        Env::<LocalDB, RandomValidator>::init(U256::ZERO, U256::ZERO, RandomValidator {})
    }

    #[fixture]
    fn rng() -> rand_xoshiro::Xoroshiro128StarStar {
        rand_xoshiro::Xoroshiro128StarStar::seed_from_u64(101)
    }

    #[rstest]
    fn test_agent_vec(
        mut env: Env<LocalDB, RandomValidator>,
        mut rng: rand_xoshiro::Xoroshiro128StarStar,
    ) {
        let a = Address::from(Uint::from(101u128));
        let b = Address::from(Uint::from(202u128));

        let agents = vec![
            TestAgent {
                address: a,
                value: 0,
            },
            TestAgent {
                address: b,
                value: 1,
            },
        ];

        let mut agent_vec = AgentVec::from(agents);

        assert_eq!(agent_vec.get_addresses(), vec![a, b]);

        agent_vec.record(&mut env);
        assert_eq!(agent_vec.records.len(), 1);

        let calls = agent_vec.call(&mut rng, &mut env);
        assert_eq!(calls.len(), 4);

        agent_vec.record(&mut env);
        assert_eq!(agent_vec.records.len(), 2);

        let records = agent_vec.take_records();

        assert_eq!(records[0], vec![0, 1]);
        assert_eq!(records[1], vec![1, 2]);
    }
}
