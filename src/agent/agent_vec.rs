use crate::agent::traits::{Agent, AgentSet, RecordedAgent, RecordedAgentSet};
use crate::contract::Call;
use crate::network::Network;
use ethers_core::types::Address;
use revm::primitives::Address as RevmAddress;
use std::any::Any;
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
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call> {
        self.agents
            .iter_mut()
            .flat_map(|x| x.update(rng, network))
            .collect()
    }
    /// Record the current state of the agents in this set.
    fn record_agents(&mut self) {
        let records: Vec<R> = self.agents.iter_mut().map(|x| x.record()).collect();
        self.records.push(records);
    }
    /// Get the revm addresses of the agents in this set.
    fn get_call_addresses(&self) -> Vec<RevmAddress> {
        self.agents.iter().map(|x| x.get_call_address()).collect()
    }
    /// Get the ethers-core addresses of the agents in this set.
    fn get_addresses(&self) -> Vec<Address> {
        self.agents.iter().map(|x| x.get_address()).collect()
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}
