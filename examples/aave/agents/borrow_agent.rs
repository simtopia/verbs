use crate::POOL_INDEX;
use ethers_core::types::{Address, U256};
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::{Agent, RecordedAgent};
use rust_sim::contract::Call;
use rust_sim::network::Network;

pub struct BorrowAgent {
    call_address: RevmAddress,
    address: Address,
    activation_rate: f32,
    has_supplied: bool,
    supply_token_address: Address,
    _borrow_token_address: Address,
}

impl BorrowAgent {
    pub fn new(
        idx: usize,
        activation_rate: f32,
        supply_token_address: Address,
        borrow_token_address: Address,
    ) -> Self {
        let idx_u64 = u64::try_from(idx).unwrap();
        let call_address = RevmAddress::from(idx_u64);
        let address = Address::from(primitive_types::H160::from_low_u64_be(idx_u64));
        BorrowAgent {
            call_address,
            address,
            activation_rate,
            has_supplied: false,
            supply_token_address,
            _borrow_token_address: borrow_token_address,
        }
    }
}

impl Agent for BorrowAgent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call> {
        if rng.f32() < self.activation_rate {
            if !self.has_supplied {
                let supply_call = network.create_call(
                    self.call_address,
                    POOL_INDEX,
                    "supply",
                    (
                        self.supply_token_address,
                        U256::from(1000),
                        self.address,
                        U256::zero(),
                    ),
                );
                self.has_supplied = true;
                Some(supply_call)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_call_address(&self) -> RevmAddress {
        self.call_address
    }

    fn get_address(&self) -> Address {
        self.address
    }
}

impl RecordedAgent<U256> for BorrowAgent {
    fn record(&self) -> U256 {
        U256::zero()
    }
}
