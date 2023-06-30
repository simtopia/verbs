use crate::agents::{BorrowAgent, LiquidationAgent};
use crate::{FAUCET_INDEX, POOL_INDEX};
use ethers_core::abi::{short_signature, ParamType};
use ethers_core::types::{Address, U256};
use rust_sim::agent::AgentVec;
use rust_sim::contract::load_contract as _load_contract;
use rust_sim::contract::ContractDefinition;
use rust_sim::network::Network;
use rust_sim::utils::convert_address;
use rust_sim::utils::inverse_convert_address;

fn load_contract(contract_path: &str, abi_path: &str, params_path: &str) -> ContractDefinition {
    _load_contract(
        format!("{}{}", contract_path, abi_path).as_str(),
        format!("{}{}", contract_path, params_path).as_str(),
        None,
    )
}

pub fn deploy_contracts(mut network: Network) -> Network {
    let contract_path = "examples/aave/contracts/";

    let contract_names =
        std::fs::File::open(format!("{}{}", contract_path, "contract_names.json")).unwrap();
    let contract_names: serde_json::Value = serde_json::from_reader(contract_names).unwrap();
    let contract_names = contract_names.as_array().unwrap();
    let contract_names: Vec<&str> = contract_names
        .into_iter()
        .map(|x| x.as_str().unwrap())
        .collect();

    for c in contract_names {
        let contract = load_contract(
            contract_path,
            format!("{}.abi", c).as_str(),
            format!("{}.json", c).as_str(),
        );
        network.deploy_contract(contract);
    }

    network
}

pub fn approve_and_mint(
    mut network: Network,
    addresses: Vec<Address>,
    token_index: usize,
    amount: u128,
) -> Network {
    let amount = U256::from(amount);
    let faucet_address = network.get_contract_address(FAUCET_INDEX);
    let pool_address = convert_address(network.get_contract_address(POOL_INDEX));

    let selector = short_signature("mint", &[ParamType::Address, ParamType::Uint(256)]);

    for address in addresses {
        network.direct_execute_with_selector(
            faucet_address,
            token_index,
            selector,
            (address, amount),
        );

        let _amount: U256 = network.direct_call(faucet_address, token_index, "balanceOf", address);

        let a = inverse_convert_address(address);
        let _approved: bool =
            network.direct_execute(a, token_index, "approve", (pool_address, U256::MAX));
    }

    network
}

pub fn initialise_borrow_agents(n_agents: usize) -> AgentVec<U256, BorrowAgent> {
    let agents = (100..100 + n_agents).map(|i| BorrowAgent::new(i)).collect();
    AgentVec::from(agents)
}

pub fn initialise_liquidation_agents(n_agents: usize) -> AgentVec<U256, LiquidationAgent> {
    let agents = (200..200 + n_agents)
        .map(|i| LiquidationAgent::new(i))
        .collect();
    AgentVec::from(agents)
}
