use crate::agent::{Agent, RecordedAgent};
use crate::conract::Transaction;
use crate::network::Network;
use ethers_core::abi::{Detokenize, Tokenize};
use kdam::tqdm;
use std::marker::PhantomData;

pub struct SimRunner<D: Detokenize, T: Tokenize, R, A: Agent<T> + RecordedAgent<R>> {
    network: Network,
    agents: Vec<A>,
    n_steps: usize,
    _marker: PhantomData<(D, T, R)>,
}

impl<D: Detokenize, T: Tokenize, R, A: Agent<T> + RecordedAgent<R>> SimRunner<D, T, R, A> {
    pub fn new(network: Network, agents: Vec<A>, n_steps: usize) -> Self {
        SimRunner {
            network: network,
            agents: agents,
            n_steps: n_steps,
            _marker: PhantomData,
        }
    }

    pub fn run(&mut self, seed: u64) -> Vec<Vec<R>> {
        let mut rng = fastrand::Rng::with_seed(seed);

        // TODO: There should be a nicer way to initialize this!
        let mut records: Vec<Vec<R>> = Vec::with_capacity(self.n_steps);

        for _ in tqdm!(0..self.n_steps) {
            let mut transactions: Vec<Transaction<T>> = (&mut self.agents)
                .into_iter()
                .map(|x| x.update(&mut rng, &mut self.network))
                .filter(|x| x.is_some())
                .map(|x| x.unwrap())
                .collect();
            rng.shuffle(transactions.as_mut_slice());
            self.network.process_transactions::<D, T>(transactions);

            records.push((&self.agents).into_iter().map(|x| x.record()).collect());
        }

        records
    }
}
