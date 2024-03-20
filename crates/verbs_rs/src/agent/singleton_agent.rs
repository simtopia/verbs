//! Implementation of [AgentSet] for a single agent

use crate::agent::traits::{Agent, AgentSet, RecordedAgent, RecordedAgentSet};
use crate::contract::Transaction;
use crate::env::{Env, Validator};
use crate::DB;
use alloy_primitives::Address;
use rand::RngCore;
use std::mem;

/// Implementation of agent set for a single agent
///
/// Convenience implementation of the [AgentSet] trait
/// for a set containing a single agent
///
/// # Examples
///
/// ```
/// use rand::RngCore;
/// use alloy_primitives::Address;
/// use verbs_rs::{DB, env::{Env, Validator}};
/// use verbs_rs::agent::{Agent, RecordedAgent, SingletonAgent, AgentSet};
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
/// let singleton_agent = SingletonAgent::<bool, DummyAgent>::from(
///     DummyAgent{}
/// );
///
/// let addresses = singleton_agent.get_addresses();
/// ```
pub struct SingletonAgent<R, A: Agent + RecordedAgent<R>> {
    /// Single agent in this set
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
    /// Call the agent and optionally return EVM call
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
        env: &mut Env<D, V>,
    ) -> Vec<Transaction> {
        self.agent.update(rng, env)
    }
    /// Record the current state of the agent
    fn record<D: DB, V: Validator>(&mut self, env: &mut Env<D, V>) {
        self.records.push(self.agent.record(env));
    }
    /// Get the address of the agent as a vector
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
