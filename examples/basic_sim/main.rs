use ethers_core::types::U256;
use rust_sim::agent::{Agent, AgentVec};
use rust_sim::contract::load_contract;
use rust_sim::network::Network;
use rust_sim::sim_runner::SimRunner;
use simple_agent::{DummyAdminAgent, SimpleAgent};
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
    let admin_address = "0x0000000000000000000000000000000000000000";

    let mut sim = Network::from_range(start_balance, 1..n_users.try_into().unwrap(), admin_address);
    sim.deploy_contract(contract);

    let admin_agent = DummyAdminAgent {};

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

    let mut agent_set = AgentVec::from(agents);

    let mut sim_runner: SimRunner<DummyAdminAgent> = SimRunner::new(sim, admin_agent, n_steps);
    sim_runner.insert_agent_set(Box::new(&mut agent_set));

    sim_runner.run(0);

    let _agent_data = agent_set.get_records();
}
