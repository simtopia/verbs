use ethers_core::types::U256;
use rust_sim::agent::{Agent, AgentVec};
use rust_sim::contract::load_contract;
use rust_sim::network::Network;
use rust_sim::sim_runner::SimRunner;
use simple_agent::SimpleAgent;
mod simple_agent;

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n_users: usize = args[1].parse::<usize>().unwrap();
    let n_steps: usize = args[2].parse::<usize>().unwrap();

    let contract_path = "./examples/basic_sim/basic_erc20_contract/";

    let contract = load_contract(
        format!("{}{}", contract_path, "basic_erc20.abi").as_str(),
        format!("{}{}", contract_path, "basic_erc20_params.json").as_str(),
        None,
    );

    let start_balance = 1000000000000u128;

    let mut sim = Network::new(start_balance, n_users);
    sim.deploy_contract(contract);

    let mut agents = Vec::<SimpleAgent>::new();

    for i in 0..n_users {
        let agent = simple_agent::SimpleAgent::new(i, n_users);
        agents.push(agent);
    }

    let start_balance = U256::from(start_balance);

    for agent in &agents {
        let _result: bool = sim.direct_execute(
            agent.get_call_address(),
            0,
            "approve",
            (agent.get_address(), start_balance),
        );
    }

    let agent_set = AgentVec::from(agents);

    let mut sim_runner: SimRunner = SimRunner::new(sim, n_steps);
    sim_runner.insert_agent_set(Box::new(agent_set));

    sim_runner.run(0);

    sim_runner.agents[0].records_to_csv("./output.csv");
}
