use crate::calls;
use ethers_core::types::{Address, U256};
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::{Agent, RecordedAgent};
use rust_sim::contract::Call;
use rust_sim::network::Network;
use rust_sim::utils::convert_address;

pub struct BorrowAgent {
    call_address: RevmAddress,
    address: Address,
    activation_rate: f32,
    borrow_token_decimals: U256,
    borrow_token_ltv: U256,
    has_supplied: bool,
    supply_token_address: Address,
    borrow_token_address: Address,
}

impl BorrowAgent {
    pub fn new(
        idx: usize,
        activation_rate: f32,
        borrow_token_ltv: U256,
        borrow_token_decimals: U256,
        supply_token_address: Address,
        borrow_token_address: Address,
    ) -> Self {
        let idx_u64 = u64::try_from(idx).unwrap();
        let call_address = RevmAddress::from(idx_u64);
        let address = convert_address(call_address);

        BorrowAgent {
            call_address,
            address,
            activation_rate,
            borrow_token_ltv,
            borrow_token_decimals,
            has_supplied: false,
            supply_token_address,
            borrow_token_address,
        }
    }
}

impl Agent for BorrowAgent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call> {
        if rng.f32() < self.activation_rate {
            if !self.has_supplied {
                let supply_call = calls::supply_call(
                    network,
                    self.address,
                    self.supply_token_address,
                    1000000000,
                );
                self.has_supplied = true;

                Some(supply_call)
            } else {
                let user_data = calls::get_user_data(network, self.address);
                let available_borrow_base = user_data.2;
                let borrow_asset_price = calls::get_asset_price(network, self.borrow_token_address);
                let exp = U256::from(10u128.pow(self.borrow_token_decimals.as_u32() - 4u32));
                let available_borrow =
                    exp * available_borrow_base * self.borrow_token_ltv / borrow_asset_price;

                if available_borrow > U256::zero() {
                    let borrow_call = calls::borrow_call(
                        network,
                        self.address,
                        self.borrow_token_address,
                        available_borrow,
                    );
                    Some(borrow_call)
                } else {
                    None
                }
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
