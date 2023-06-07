use ethers_core::types::{Address, U256};
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::{Agent, RecordedAgent};
use rust_sim::conract::Transaction;
use rust_sim::network::Network;

pub struct SimpleAgent {
    pub call_address: RevmAddress,
    pub address: Address,
    current_balance: U256,
    n_agents: u64,
}

impl SimpleAgent {
    pub fn new(idx: usize, n_agents: usize) -> Self {
        let idx_u64 = u64::try_from(idx).unwrap();
        let call_address = RevmAddress::from(idx_u64);
        let address = Address::from(primitive_types::H160::from_low_u64_be(idx_u64));
        SimpleAgent {
            call_address,
            address,
            current_balance: U256::zero(),
            n_agents: u64::try_from(n_agents).unwrap(),
        }
    }
}

impl Agent<(Address, U256)> for SimpleAgent {
    fn update(
        &mut self,
        rng: &mut Rng,
        network: &mut Network,
    ) -> Option<Transaction<(Address, U256)>> {
        self.current_balance =
            network.call_without_commit(self.call_address, 0, "balanceOf", (self.address,));

        if self.current_balance > U256::from(0u64) {
            let receiver = rng.u64(0..self.n_agents);
            let receiver = Address::from(primitive_types::H160::from_low_u64_be(receiver));
            let send_amount = std::cmp::min(self.current_balance, U256::from(1000));
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

impl RecordedAgent<U256> for SimpleAgent {
    fn record(&self) -> U256 {
        self.current_balance
    }
}
