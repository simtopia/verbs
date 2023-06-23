use crate::agent::traits::{Agent, AgentSet, RecordedAgent};
use crate::contract::Call;
use crate::network::Network;
use crate::utils::csv_writer;
use ethers_core::types::Address;
use revm::primitives::Address as RevmAddress;

pub struct AgentVec<R: ToString, A: Agent + RecordedAgent<R>> {
    agents: Vec<A>,
    records: Vec<Vec<R>>,
}

impl<R: ToString, A: Agent + RecordedAgent<R>> AgentVec<R, A> {
    pub fn new() -> Self {
        AgentVec {
            agents: Vec::<A>::new(),
            records: Vec::<Vec<R>>::new(),
        }
    }
    pub fn from(agents: Vec<A>) -> Self {
        AgentVec {
            agents,
            records: Vec::<Vec<R>>::new(),
        }
    }
    pub fn add_agent(&mut self, agent: A) {
        self.agents.push(agent);
    }
}

impl<R: ToString, A: Agent + RecordedAgent<R>> AgentSet for AgentVec<R, A> {
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call> {
        (&mut self.agents)
            .into_iter()
            .map(|x| x.update(rng, network))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }
    fn record_agents(&mut self) {
        let records: Vec<R> = (&mut self.agents).into_iter().map(|x| x.record()).collect();
        self.records.push(records);
    }
    fn records_to_csv(&self, output_path: &str) {
        csv_writer::<R>(&self.records, output_path);
    }
    fn get_call_addresses(&self) -> Vec<RevmAddress> {
        self.agents.iter().map(|x| x.get_call_address()).collect()
    }
    fn get_addresses(&self) -> Vec<Address> {
        self.agents.iter().map(|x| x.get_address()).collect()
    }
}