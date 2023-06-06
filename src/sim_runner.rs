use crate::agent::Agent;
use crate::network::SimulationEnvironment;
use ethers_core::abi::Tokenize;
use kdam::tqdm;
use std::marker::PhantomData;

pub struct SimRunner<T: Tokenize, A: Agent<T>> {
    network: SimulationEnvironment,
    agents: Vec<A>,
    n_steps: u64,
    _marker: PhantomData<T>,
}

impl<T: Tokenize, A: Agent<T>> SimRunner<T, A> {
    pub fn new(network: SimulationEnvironment, agents: Vec<A>, n_steps: u64) -> Self {
        SimRunner {
            network: network,
            agents: agents,
            n_steps: n_steps,
            _marker: PhantomData,
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
