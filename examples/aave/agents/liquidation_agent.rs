use crate::calls;
use ethers_core::types::{Address, U256};
use fastrand::Rng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::{Agent, RecordedAgent};
use rust_sim::contract::Call;
use rust_sim::network::Network;

pub struct LiquidationAgent {
    call_address: RevmAddress,
    address: Address,
    collateral_token_address: Address,
    debt_token_address: Address,
    liquidation_addresses: Vec<Address>,
}

fn health_factor(network: &mut Network, address: Address) -> U256 {
    let user_data = calls::get_user_data(network, address);
    user_data.5
}

impl LiquidationAgent {
    pub fn new(
        idx: usize,
        collateral_token_address: Address,
        debt_token_address: Address,
        liquidation_addresses: Vec<Address>,
    ) -> Self {
        let idx_u64 = u64::try_from(idx).unwrap();
        let call_address = RevmAddress::from(idx_u64);
        let address = Address::from(primitive_types::H160::from_low_u64_be(idx_u64));

        LiquidationAgent {
            call_address,
            address,
            collateral_token_address,
            debt_token_address,
            liquidation_addresses,
        }
    }
}

impl Agent for LiquidationAgent {
    fn update(&mut self, rng: &mut Rng, network: &mut Network) -> Option<Call> {
        // TODO: Can calculate amount to cover using this
        //  https://docs.aave.com/developers/guides/liquidations#executing-the-liquidation-call
        let health_factors: Vec<(Address, U256)> = self
            .liquidation_addresses
            .iter()
            .map(|x| (x.clone(), health_factor(network, x.clone())))
            .filter(|x| x.1 < U256::from(1000u128))
            .collect();

        let n_liquidation = health_factors.len();

        if n_liquidation > 0 {
            let i = rng.usize(0..n_liquidation);
            let selection = health_factors[i];
            let c = calls::liquidation_call(
                network,
                self.collateral_token_address,
                self.debt_token_address,
                selection.0,
                self.address,
                U256::MAX,
            );
            Some(c)
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

impl RecordedAgent<U256> for LiquidationAgent {
    fn record(&self) -> U256 {
        U256::zero()
    }
}
