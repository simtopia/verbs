use crate::conract::Transaction;
use crate::network::Network;
use ethers_core::abi::Tokenize;
use fastrand::Rng;

pub trait Agent<T: Tokenize> {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Transaction<T>>;
}

pub trait RecordedAgent<T> {
    fn record(&self) -> T;
}
