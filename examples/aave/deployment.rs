use crate::agents::{BorrowAgent, LiquidationAgent};
use crate::{FAUCET_INDEX, POOL_INDEX};
use ethers_core::types::{Address, U256};
use rust_sim::agent::AgentVec;
use rust_sim::network::Network;
use rust_sim::utils::Cast;

pub fn admin_mint_and_supply(mut network: Network, token_index: usize, amount: u128) -> Network {
    let amount = U256::from(amount);
    let faucet_address = network.get_contract_address(FAUCET_INDEX).cast();
    let pool_address = network.get_contract_address(POOL_INDEX);
    let token_address = network.get_contract_address(token_index);
    let admin_address = network.admin_address.cast();

    let _minted: U256 = network.direct_execute(
        faucet_address,
        FAUCET_INDEX,
        "mint",
        (token_address, admin_address, amount),
    );

    let _approved: bool = network.direct_execute(
        network.admin_address,
        token_index,
        "approve",
        (pool_address, U256::MAX),
    );

    let _: () = network.direct_execute(
        network.admin_address,
        POOL_INDEX,
        "supply",
        (token_address, amount, admin_address, U256::zero()),
    );

    network
}

pub fn approve_and_mint(
    mut network: Network,
    addresses: Vec<Address>,
    token_index: usize,
    amount: u128,
) -> Network {
    let amount = U256::from(amount);
    let faucet_address = network.get_contract_address(FAUCET_INDEX).cast();
    let pool_address = network.get_contract_address(POOL_INDEX);
    let token_address = network.get_contract_address(token_index);

    for address in addresses {
        let _minted: U256 = network.direct_execute(
            faucet_address,
            FAUCET_INDEX,
            "mint",
            (token_address, address, amount),
        );
        let _amount: U256 = network.direct_call(faucet_address, token_index, "balanceOf", address);
        let a = address.cast();
        let _approved: bool =
            network.direct_execute(a, token_index, "approve", (pool_address, U256::MAX));
    }

    network
}

pub fn initialise_borrow_agents(
    n_agents: usize,
    activation_rate: f32,
    borrow_token_ltv: U256,
    borrow_token_decimals: U256,
    supply_token_address: Address,
    borrow_token_address: Address,
) -> AgentVec<U256, BorrowAgent> {
    let agents = (100..100 + n_agents)
        .map(|i| {
            BorrowAgent::new(
                i,
                activation_rate,
                borrow_token_ltv,
                borrow_token_decimals,
                supply_token_address,
                borrow_token_address,
            )
        })
        .collect();
    AgentVec::from(agents)
}

pub fn initialise_liquidation_agents(
    n_agents: usize,
    supply_token_address: Address,
    borrow_token_address: Address,
    liquidation_addresses: Vec<Address>,
) -> AgentVec<U256, LiquidationAgent> {
    let agents = (200..200 + n_agents)
        .map(|i| {
            LiquidationAgent::new(
                i,
                supply_token_address,
                borrow_token_address,
                liquidation_addresses.clone(),
            )
        })
        .collect();
    AgentVec::from(agents)
}
