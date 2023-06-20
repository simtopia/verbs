use crate::conract::Call;
use crate::network::Network;
use crate::utils::csv_writer;
use ethers_core::types::Address;
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;
use std::fmt::Display;

pub trait Agent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call>;
    fn get_call_address(&self) -> RevmAddress;
    fn get_address(&self) -> Address;
}

pub trait RecordedAgent<R> {
    fn record(&self) -> R;
}

pub trait UpdateAgents {
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call>;
    fn record_agents(&mut self);
    fn records_to_csv(&self, path: String);
}

pub struct AgentSet<R: Display, A: Agent + RecordedAgent<R>> {
    agents: Vec<A>,
    records: Vec<Vec<R>>,
}

impl<R: Display, A: Agent + RecordedAgent<R>> AgentSet<R, A> {
    pub fn new() -> Self {
        AgentSet {
            agents: Vec::<A>::new(),
            records: Vec::<Vec<R>>::new(),
        }
    }
    pub fn from(agents: Vec<A>) -> Self {
        AgentSet {
            agents,
            records: Vec::<Vec<R>>::new(),
        }
    }
    pub fn add_agent(&mut self, agent: A) {
        self.agents.push(agent);
    }
}

impl<R: Display, A: Agent + RecordedAgent<R>> UpdateAgents for AgentSet<R, A> {
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
    fn records_to_csv(&self, output_path: String) {
        csv_writer::<R>(&self.records, output_path);
    }
}
