use crate::ecr20;
use alloy_primitives::{Address, Uint, U256};
use alloy_sol_types::SolCall;
use fastrand::Rng;
use rust_sim::agent::{AdminAgent, Agent, RecordedAgent};
use rust_sim::contract::Call;
use rust_sim::network::Network;

pub struct DummyAdminAgent {}

impl AdminAgent for DummyAdminAgent {
    fn update(&mut self, _rng: &mut Rng, _network: &mut Network) {}
    fn post_update(&mut self, _network: &mut Network) {}
}

pub struct SimpleAgent {
    pub address: Address,
    current_balance: U256,
    n_agents: u64,
    token_address: Address,
}

impl SimpleAgent {
    pub fn new(idx: usize, n_agents: usize, token_address: Address) -> Self {
        let idx_u64 = u64::try_from(idx).unwrap();
        let address = Address::from(Uint::from(idx_u64));
        SimpleAgent {
            address,
            current_balance: U256::ZERO,
            n_agents: u64::try_from(n_agents).unwrap(),
            token_address,
        }
    }
}

impl Agent for SimpleAgent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Vec<Call> {
        self.current_balance = network
            .direct_call(
                self.address,
                self.token_address,
                "balanceOf",
                ecr20::ABI::balanceOfCall {
                    tokenOwner: self.address,
                },
            )
            .unwrap()
            .0
            .balance;

        if self.current_balance > U256::from(0u64) {
            let receiver = rng.u64(0..self.n_agents);
            let receiver = Address::from(Uint::from(receiver));
            let send_amount = std::cmp::min(self.current_balance, U256::from(1000));
            let send_call = Call {
                function_name: "transfer",
                callee: self.address,
                transact_to: self.token_address,
                args: ecr20::ABI::transferCall {
                    to: receiver,
                    tokens: send_amount,
                }
                .abi_encode(),
                checked: true,
            };
            vec![send_call]
        } else {
            Vec::default()
        }
    }

    fn get_address(&self) -> Address {
        self.address
    }
}

impl RecordedAgent<U256> for SimpleAgent {
    fn record(&mut self) -> U256 {
        self.current_balance
    }
}
