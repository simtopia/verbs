use crate::conract::Call;
use crate::network::Network;
use ethers_core::types::Address;
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;

pub trait Agent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call>;
    fn get_call_address(&self) -> RevmAddress;
    fn get_address(&self) -> Address;
}

pub trait RecordedAgent<T> {
    fn record(&self) -> T;
}
