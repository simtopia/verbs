mod ecr20;
mod simple_agent;

use alloy_primitives::U256;
use rust_sim::agent::AgentVec;
use rust_sim::network::Network;
use rust_sim::sim_runner::SimRunner;
use rust_sim::utils;
use simple_agent::{DummyAdminAgent, SimpleAgent};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n_users: usize = args[1].parse::<usize>().unwrap();
    let n_steps: usize = args[2].parse::<usize>().unwrap();

    let start_balance = 1000000000000u128;
    let admin_address = "0x1000000000000000000000000000000000000000";

    let mut sim = Network::from_range(start_balance, 1..n_users.try_into().unwrap(), admin_address);

    let token_address =
        sim.manually_deploy_contract("ECR20", utils::data_bytes_from_hex(ecr20::BYTECODE));

    let admin_agent = DummyAdminAgent {};

    let mut agents = Vec::<SimpleAgent>::new();

    for i in 0..n_users {
        let agent = SimpleAgent::new(i, n_users, token_address);
        agents.push(agent);
    }

    let start_balance = U256::from(start_balance);

    for agent in &agents {
        sim.direct_execute(
            agent.address,
            token_address,
            "approve",
            ecr20::ABI::approveCall {
                spender: agent.address,
                tokens: start_balance,
            },
        )
        .unwrap();
    }

    let agent_set = AgentVec::from(agents);

    let mut sim_runner: SimRunner<DummyAdminAgent> = SimRunner::new(sim, admin_agent);
    sim_runner.insert_agent_set(agent_set);

    sim_runner.run(0, n_steps);

    let _agent_data = sim_runner
        .agents
        .get_records::<U256, AgentVec<U256, simple_agent::SimpleAgent>>(0);
}
