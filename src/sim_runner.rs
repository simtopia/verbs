use crate::agent::Agent;
use crate::conract::Call;
use crate::network::Network;
use kdam::tqdm;

pub struct SimRunner {
    network: Network,
    agents: Vec<Box<dyn Agent>>,
    n_steps: usize,
}

impl SimRunner {
    pub fn new(network: Network, agents: Vec<Box<dyn Agent>>, n_steps: usize) -> Self {
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
                .map(|x| x.update(&mut rng, n))
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect();
            rng.shuffle(calls.as_mut_slice());
            self.network.process_calls(calls);
        }
    }
}
