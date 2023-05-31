use ethers_core::types::{Address, U256};
use rust_sim::conract::ContractDefinition;
use rust_sim::network::SimulationEnvironment;

fn main() {
    let contract_abi_path = String::from("./contracts/basic_erc20_contract/basic_erc20.abi");
    let contract_params_path =
        String::from("./contracts/basic_erc20_contract/basic_erc20_params.json");

    let contract = ContractDefinition::load(contract_abi_path, contract_params_path);

    let start_balance: u128 = 1000000000000;

    let mut sim = SimulationEnvironment::new(start_balance, 2);
    sim.deploy_contract(contract);

    let _result: bool = sim.call_contract(
        1,
        0,
        "approve",
        (
            Address::from(primitive_types::H160::from_low_u64_be(1u64)),
            U256::from(1000000000),
        ),
    );

    let _result: bool = sim.call_contract(
        0,
        0,
        "transfer",
        (
            Address::from(primitive_types::H160::from_low_u64_be(1u64)),
            U256::from(100),
        ),
    );

    let _result: U256 = sim.call_contract(
        0,
        0,
        "balanceOf",
        (Address::from(primitive_types::H160::from_low_u64_be(1u64)),),
    );

    println!("{}", _result);

    let _result: U256 = sim.call_contract(
        0,
        0,
        "balanceOf",
        (Address::from(primitive_types::H160::from_low_u64_be(0u64)),),
    );

    println!("{}", _result);
}
