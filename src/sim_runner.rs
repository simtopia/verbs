use crate::agent::{traits::AdminAgent, AgentSet};
use crate::contract::Call;
use crate::network::Network;
use kdam::tqdm;

pub type AgentSetRef<'a> = Box<&'a mut dyn AgentSet>;
pub type AgentSetVec<'a> = Vec<AgentSetRef<'a>>;

pub trait AgentSetVecMethods<'a> {
    fn push_agent_set<A: AgentSet>(&mut self, agent_set: &'a mut A);
}

impl<'a> AgentSetVecMethods<'a> for AgentSetVec<'a> {
    fn push_agent_set<A: AgentSet>(&mut self, agent_set: &'a mut A) {
        self.push(Box::new(agent_set))
    }
}

pub struct SimRunner<'a, A: AdminAgent> {
    network: Network,
    pub admin_agent: A,
    pub agents: AgentSetVec<'a>,
    n_steps: usize,
}

impl<'a, A: AdminAgent> SimRunner<'a, A> {
    pub fn new(network: Network, admin_agent: A, n_steps: usize) -> Self {
        SimRunner {
            network,
            admin_agent,
            agents: AgentSetVec::new(),
            n_steps,
        }
    }

    pub fn from_agents(
        network: Network,
        admin_agent: A,
        n_steps: usize,
        agents: AgentSetVec<'a>,
    ) -> Self {
        SimRunner {
            network,
            admin_agent,
            agents,
            n_steps,
        }
    }

    pub fn insert_agent_set<S: AgentSet>(&mut self, agent_set: &'a mut S) {
        self.agents.push(Box::new(agent_set));
    }

    pub fn run(&mut self, seed: u64) {
        let mut rng = fastrand::Rng::with_seed(seed);

        for _ in tqdm!(0..self.n_steps) {
            let n = &mut self.network;

            self.admin_agent.update(&mut rng, n);

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
