use ethers_core::types::U256;
use rust_sim::conract::{ContractDefinition, Transaction};
use rust_sim::network::Network;
use rust_sim::sim_runner::SimRunner;
use rust_sim::utils;
mod simple_agent;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n_users: usize = args[1].parse::<usize>().unwrap();
    let n_steps: usize = args[2].parse::<usize>().unwrap();

    let contract_path = String::from("./examples/basic_sim/basic_erc20_contract/");

    let contract_abi_path = format!("{}{}", contract_path, String::from("basic_erc20.abi"));
    let contract_params_path = format!(
        "{}{}",
        contract_path,
        String::from("basic_erc20_params.json")
    );

    let contract = ContractDefinition::load(contract_abi_path, contract_params_path, None);

    let start_balance = 1000000000000u128;

    let mut sim = Network::new(start_balance, n_users);
    sim.deploy_contract(contract);

    let mut agents = Vec::<simple_agent::SimpleAgent>::new();

    for i in 0..n_users {
        let agent = simple_agent::SimpleAgent::new(i, n_users);
        agents.push(agent);
    }

    let start_balance = U256::from(start_balance);
    for agent in &agents {
        let result_call = Transaction {
            callee: agent.call_address,
            function_name: "approve",
            contract_idx: 0,
            args: (agent.address, start_balance),
        };
        let _result: bool = sim.call_contract(result_call);
    }

    let mut sim_runner: SimRunner<bool, U256, simple_agent::SimpleAgent> =
        SimRunner::new(sim, agents, n_steps);

    let records = sim_runner.run(0);

    utils::csv_writer(records, String::from("./output.csv"));
}
