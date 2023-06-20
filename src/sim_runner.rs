use crate::agent::Agent;
use crate::conract::Call;
use crate::network::Network;
use kdam::tqdm;

pub trait UpdateAgents {
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call>;
}

pub struct AgentSet<A: Agent> {
    agents: Vec<A>,
}

impl<A: Agent> AgentSet<A> {
    pub fn new() -> Self {
        AgentSet {
            agents: Vec::<A>::new(),
        }
    }
    pub fn from(agents: Vec<A>) -> Self {
        AgentSet { agents }
    }
    pub fn add_agent(&mut self, agent: A) {
        self.agents.push(agent);
    }
}

impl<A: Agent> UpdateAgents for AgentSet<A> {
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call> {
        (&mut self.agents)
            .into_iter()
            .map(|x| x.update(rng, network))
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect()
    }
}

pub struct SimRunner {
    network: Network,
    agents: Vec<Box<dyn UpdateAgents>>,
    n_steps: usize,
}

impl SimRunner {
    pub fn new(network: Network, agents: Vec<Box<dyn UpdateAgents>>, n_steps: usize) -> Self {
        SimRunner {
            network: network,
            agents: agents,
            n_steps: n_steps,
        }
    }

    pub fn run(&mut self, seed: u64) {
        let mut rng = fastrand::Rng::with_seed(seed);

        for _ in tqdm!(0..self.n_steps) {
            let n = &mut self.network;
            let mut calls: Vec<Call> = (&mut self.agents)
                .into_iter()
                .map(|x| x.call_agents(&mut rng, n))
                .flatten()
                .collect();
            rng.shuffle(calls.as_mut_slice());
            self.network.process_calls(calls);
        }
    }
}
