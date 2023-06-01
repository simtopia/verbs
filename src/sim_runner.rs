use crate::agent::Agent;
use crate::network::SimulationEnvironment;
use kdam::tqdm;

pub struct SimRunner<A: Agent> {
    network: SimulationEnvironment,
    agents: Vec<A>,
    n_steps: u64,
}

impl<A: Agent> SimRunner<A> {
    pub fn new(network: SimulationEnvironment, agents: Vec<A>, n_steps: u64) -> Self {
        SimRunner {
            network: network,
            agents: agents,
            n_steps: n_steps,
        }
    }

    pub fn run(&mut self) {
        let mut rng = rand::thread_rng();

        for _ in tqdm!(0..self.n_steps) {
            for agent in &mut self.agents {
                agent.update(&mut rng, &mut self.network);
            }
        }
    }
}
