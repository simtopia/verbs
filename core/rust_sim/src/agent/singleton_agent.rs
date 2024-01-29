use crate::agent::traits::{Agent, AgentSet, RecordedAgent, RecordedAgentSet};
use crate::contract::Transaction;
use crate::network::Network;
use alloy_primitives::Address;
use fork_evm::DB;
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
        mem::take(&mut self.records)
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
    fn call<D: DB>(
        &mut self,
        rng: &mut fastrand::Rng,
        network: &mut Network<D>,
    ) -> Vec<Transaction> {
        self.agent.update(rng, network)
    }
    /// Record the current state of the agent.
    fn record(&mut self) {
        self.records.push(self.agent.record());
    }
    /// Get the ethers-core addresses of the agent.
    fn get_addresses(&self) -> Vec<Address> {
        vec![self.agent.get_address()]
    }
}

impl<R, A: Agent + RecordedAgent<R>> RecordedAgentSet<R> for SingletonAgent<R, A> {
    fn take_records(&mut self) -> Vec<Vec<R>> {
        let x = mem::take(&mut self.records);
        x.into_iter().map(|y| vec![y]).collect()
    }
}
