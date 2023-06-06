use crate::conract::Transaction;
use crate::network::Network;
use ethers_core::abi::Tokenize;
use rand::rngs::ThreadRng;

pub trait Agent<T: Tokenize> {
    fn update(&mut self, rng: &mut ThreadRng, network: &mut Network) -> Option<Transaction<T>>;
}
