use ethers_core::types::{Address, U256};
use fastrand::Rng;
use rust_sim::agent::AdminAgent;
use rust_sim::network::Network;

pub struct PriceAdminAgent {
    pub token_a_price: U256,
    pub token_b_price: U256,
    pub token_a_address: Address,
    pub token_b_address: Address,
}

impl PriceAdminAgent {
    pub fn new(
        token_a_price: u128,
        token_b_price: u128,
        token_a_address: Address,
        token_b_address: Address,
    ) -> Self {
        PriceAdminAgent {
            token_a_price: U256::from(token_a_price),
            token_b_price: U256::from(token_a_price),
            token_a_address,
            token_b_address,
        }
    }
}

impl AdminAgent for PriceAdminAgent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) {}
}
