use crate::agent::Agent;
use crate::conract::Transaction;
use crate::network::Network;
use ethers_core::abi::{Detokenize, Tokenize};
use kdam::tqdm;
use std::marker::PhantomData;

pub struct SimRunner<D: Detokenize, T: Tokenize, A: Agent<T>> {
    network: Network,
    agents: Vec<A>,
    n_steps: u64,
    _marker: PhantomData<(D, T)>,
}

impl<D: Detokenize, T: Tokenize, A: Agent<T>> SimRunner<D, T, A> {
    pub fn new(network: Network, agents: Vec<A>, n_steps: u64) -> Self {
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
            let transactions: Vec<Transaction<T>> = (&mut self.agents)
                .into_iter()
                .map(|x| x.update(&mut rng, &mut self.network))
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect();

            self.network.process_transactions::<D, T>(transactions)
        }
    }
}
