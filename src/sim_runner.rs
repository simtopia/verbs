use crate::agent::AgentSet;
use crate::contract::Call;
use crate::network::Network;
use kdam::tqdm;

pub struct SimRunner {
    network: Network,
    pub agents: Vec<Box<dyn AgentSet>>,
    n_steps: usize,
}

impl SimRunner {
    pub fn new(network: Network, n_steps: usize) -> Self {
        SimRunner {
            network,
            agents: Vec::<Box<dyn AgentSet>>::new(),
            n_steps,
        }
    }

    pub fn from_agents(network: Network, n_steps: usize, agents: Vec<Box<dyn AgentSet>>) -> Self {
        SimRunner {
            network,
            agents,
            n_steps,
        }
    }

    pub fn insert_agent_set(&mut self, agent_set: Box<dyn AgentSet>) {
        self.agents.push(agent_set);
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
            for agent_set in &mut self.agents {
                agent_set.record_agents();
            }
        }
    }
}
