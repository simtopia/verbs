use crate::contract::Call;
use crate::network::Network;
use ethers_core::types::Address;
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;

pub trait Agent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call>;
    fn get_call_address(&self) -> RevmAddress;
    fn get_address(&self) -> Address;
}

pub trait RecordedAgent<R> {
    fn record(&self) -> R;
}

pub trait AgentSet {
    fn call_agents(&mut self, rng: &mut fastrand::Rng, network: &mut Network) -> Vec<Call>;
    fn record_agents(&mut self);
    fn records_to_csv(&self, path: &str);
    fn get_call_addresses(&self) -> Vec<RevmAddress>;
    fn get_addresses(&self) -> Vec<Address>;
}
