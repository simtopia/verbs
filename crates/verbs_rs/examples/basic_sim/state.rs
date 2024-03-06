use crate::ecr20;
use alloy_primitives::{Address, Uint, U256};
use rand::Rng;
use rand::RngCore;
use verbs_rs::agent::{Agent, AgentSet, AgentVec, RecordedAgent, SimState};
use verbs_rs::contract::Transaction;
use verbs_rs::env::Env;
use verbs_rs::DB;

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
    fn update<D: DB, R: RngCore>(&mut self, rng: &mut R, network: &mut Env<D>) -> Vec<Transaction> {
        self.current_balance = network
            .direct_call(
                self.address,
                self.token_address,
                ecr20::ABI::balanceOfCall {
                    tokenOwner: self.address,
                },
                U256::ZERO,
            )
            .unwrap()
            .0
            .balance;

        if self.current_balance > U256::from(0u64) {
            let receiver = rng.gen_range(0..self.n_agents);
            let receiver = Address::from(Uint::from(receiver));
            let send_amount = std::cmp::min(self.current_balance, U256::from(1000));
            let send_call = Transaction::new(
                self.address,
                self.token_address,
                ecr20::ABI::transferCall {
                    to: receiver,
                    tokens: send_amount,
                },
                U256::ZERO,
                true,
            );
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
    fn record<D: DB>(&mut self, _env: &mut Env<D>) -> U256 {
        self.current_balance
    }
}

#[derive(SimState)]
pub struct AgentState {
    pub agents: AgentVec<U256, SimpleAgent>,
}
