use ethers_core::types::{Address, U256};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::Agent;
use rust_sim::conract::Transaction;
use rust_sim::network::Network;

pub struct SimpleAgent {
    pub call_address: RevmAddress,
    pub address: Address,
    dist: Uniform<u64>,
}

impl SimpleAgent {
    pub fn new(idx: u64, n_agents: u64) -> Self {
        let call_address = RevmAddress::from(idx);
        let address = Address::from(primitive_types::H160::from_low_u64_be(idx));
        let dist = Uniform::from(0..n_agents);
        SimpleAgent {
            call_address,
            address,
            dist,
        }
    }
}

impl Agent<(Address, U256)> for SimpleAgent {
    fn update(
        &mut self,
        rng: &mut ThreadRng,
        network: &mut Network,
    ) -> Option<Transaction<(Address, U256)>> {
        let balance_call = Transaction {
            callee: self.call_address,
            function_name: "balanceOf",
            contract_idx: 0,
            args: (self.address,),
        };
        let current_balance: U256 = network.call_contract(balance_call);

        if current_balance > U256::from(0u64) {
            let receiver = self.dist.sample(rng);
            let receiver = Address::from(primitive_types::H160::from_low_u64_be(receiver));
            let send_amount = std::cmp::min(current_balance, U256::from(1000));
            let send_call = Transaction {
                callee: self.call_address,
                function_name: "transfer",
                contract_idx: 0,
                args: (receiver, send_amount),
            };
            Some(send_call)
        } else {
            None
        }
    }
}
