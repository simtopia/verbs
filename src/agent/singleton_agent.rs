use crate::agent::traits::{Agent, AgentSet, RecordedAgent, RecordedAgentSet};
use crate::contract::Call;
use crate::network::Network;
use ethers_core::types::Address;
use revm::primitives::Address as RevmAddress;
use std::any::Any;
use std::mem;

/// Implementation of agent set for a single agent.
pub struct SingletonAgent<R, A: Agent + RecordedAgent<R>> {
    /// Single agent in this set.
    agent: A,
    /// Records of agent state over the course of the simulation.
    records: Vec<R>,
}

impl<R, A: Agent + RecordedAgent<R>> SingletonAgent<R, A> {
    /// Initialise an from an existing agents.
    ///
    /// # Arguments
    ///
    /// * `agent` - Agent controlled by the agent-set
    ///
    pub fn from(agent: A) -> Self {
        SingletonAgent {
            agent,
            records: Vec::<R>::new(),
        }
    }
    /// Get the recorded history of the agent.
    pub fn get_records(&self) -> &Vec<R> {
        &self.records
    }

    /// Take ownership of recorded data of the agent.
    pub fn take_records(&mut self) -> Vec<R> {
        mem::replace(&mut self.records, Vec::new())
    }
}

/// Implementations of agent updates and recording.
impl<R: 'static, A: Agent + RecordedAgent<R> + 'static> AgentSet for SingletonAgent<R, A> {
    /// Call the agent and optionally return EVM call.
    ///
    /// # Arguments
    ///
    /// * `rng` - Fastrand rng state
    /// * `network` - Protocol deployment(s)
    ///
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call> {
        self.agent.update(rng, network)
    }
    /// Record the current state of the agent.
    fn record_agents(&mut self) {
        self.records.push(self.agent.record());
    }
    /// Get the revm addresses of the agent.
    fn get_call_addresses(&self) -> Vec<RevmAddress> {
        vec![self.agent.get_call_address()]
    }
    /// Get the ethers-core addresses of the agent.
    fn get_addresses(&self) -> Vec<Address> {
        vec![self.agent.get_address()]
    }
    fn as_mut_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl<R, A: Agent + RecordedAgent<R>> RecordedAgentSet<R> for SingletonAgent<R, A> {
    fn take_records(&mut self) -> Vec<Vec<R>> {
        let x = mem::replace(&mut self.records, Vec::new());
        x.into_iter().map(|y| vec![y]).collect()
    }
}
