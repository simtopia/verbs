use crate::agent::traits::{Agent, AgentSet, RecordedAgent, RecordedAgentSet};
use crate::contract::Call;
use crate::network::Network;
use alloy_primitives::Address;
use revm::db::DatabaseRef;
use std::mem;

/// Implementation of agent set tracking agents as a vector.
pub struct AgentVec<R, A: Agent + RecordedAgent<R>> {
    /// Vector of agents of a single type.
    agents: Vec<A>,
    /// Records of agent states over the course of the simulation.
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
    /// Initialise an empty vector agent-set.
    pub fn new() -> Self {
        AgentVec {
            agents: Vec::<A>::new(),
            records: Vec::<Vec<R>>::new(),
        }
    }
    /// Initialise an agent-set from an existing vector of agents.
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
    fn take_records(&mut self) -> Vec<Vec<R>> {
        mem::take(&mut self.records)
    }
}

/// Implementations of agent updates and recording.
impl<R: 'static, A: Agent + RecordedAgent<R> + 'static> AgentSet for AgentVec<R, A> {
    /// Call the agents in the set and collect any returned EVM calls.
    ///
    /// # Arguments
    ///
    /// * `rng` - Fastrand rng state
    /// * `network` - Protocol deployment(s)
    ///
    fn call<D: DatabaseRef>(
        &mut self,
        rng: &mut fastrand::Rng,
        network: &mut Network<D>,
    ) -> Vec<Call> {
        self.agents
            .iter_mut()
            .flat_map(|x| x.update(rng, network))
            .collect()
    }
    /// Record the current state of the agents in this set.
    fn record(&mut self) {
        let records: Vec<R> = self.agents.iter_mut().map(|x| x.record()).collect();
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
    use crate::agent::traits;
    use alloy_primitives::Uint;
    use revm::db::EmptyDB;
    use rstest::*;

    struct TestAgent {
        address: Address,
        value: u64,
    }

    impl traits::Agent for TestAgent {
        fn update<D: DatabaseRef>(
            &mut self,
            _rng: &mut fastrand::Rng,
            _network: &mut crate::network::Network<D>,
        ) -> Vec<crate::contract::Call> {
            self.value += 1;
            vec![
                Call {
                    function_name: "foo",
                    callee: Address::ZERO,
                    transact_to: Address::ZERO,
                    args: Vec::default(),
                    checked: false,
                },
                Call {
                    function_name: "foo",
                    callee: Address::ZERO,
                    transact_to: Address::ZERO,
                    args: Vec::default(),
                    checked: false,
                },
            ]
        }

        fn get_address(&self) -> Address {
            self.address
        }
    }

    impl traits::RecordedAgent<u64> for TestAgent {
        fn record(&mut self) -> u64 {
            self.value
        }
    }

    #[fixture]
    fn network() -> Network<EmptyDB> {
        Network::<EmptyDB>::init(Address::from(Uint::from(999)).to_string().as_str())
    }

    #[fixture]
    fn rng() -> fastrand::Rng {
        fastrand::Rng::default()
    }

    #[rstest]
    fn test_agent_vec(mut network: Network<EmptyDB>, mut rng: fastrand::Rng) {
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

        agent_vec.record();
        assert_eq!(agent_vec.records.len(), 1);

        let calls = agent_vec.call(&mut rng, &mut network);
        assert_eq!(calls.len(), 4);

        agent_vec.record();
        assert_eq!(agent_vec.records.len(), 2);

        let records = agent_vec.take_records();

        assert_eq!(records[0], vec![0, 1]);
        assert_eq!(records[1], vec![1, 2]);
    }
}