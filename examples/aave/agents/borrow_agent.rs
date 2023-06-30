use ethers_core::types::{Address, U256};
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::{Agent, RecordedAgent};
use rust_sim::contract::Call;
use rust_sim::network::Network;

pub struct BorrowAgent {
    pub call_address: RevmAddress,
    pub address: Address,
}

impl BorrowAgent {
    pub fn new(idx: usize) -> Self {
        let idx_u64 = u64::try_from(idx).unwrap();
        let call_address = RevmAddress::from(idx_u64);
        let address = Address::from(primitive_types::H160::from_low_u64_be(idx_u64));
        BorrowAgent {
            call_address,
            address,
        }
    }
}

impl Agent for BorrowAgent {
    fn update(&mut self, _rng: &mut Rng, _network: &mut Network) -> Option<Call> {
        None
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
