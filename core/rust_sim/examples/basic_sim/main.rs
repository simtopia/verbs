mod ecr20;
mod state;

use alloy_primitives::{Address, Uint, U256};
use rust_sim::agent::AgentVec;
use rust_sim::network::Network;
use rust_sim::sim_runner::run;
use rust_sim::utils;
use state::{AgentState, DummyAdminAgent, SimpleAgent};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n_users: usize = args[1].parse::<usize>().unwrap();
    let n_steps: usize = args[2].parse::<usize>().unwrap();

    let start_balance = 1000000000000u128;
    let admin_address = Address::from(Uint::from(999));

    let mut sim = Network::from_range(start_balance, 1..n_users.try_into().unwrap());

    let token_address = sim.deploy_contract(
        admin_address,
        "ECR20",
        utils::constructor_data(ecr20::BYTECODE, None),
    );

    let mut admin_agent = DummyAdminAgent {};

    let agents: Vec<SimpleAgent> = (0..n_users)
        .into_iter()
        .map(|x| SimpleAgent::new(x, n_users, token_address))
        .collect();

    let start_balance = U256::from(start_balance);

    for agent in &agents {
        sim.direct_execute(
            agent.address,
            token_address,
            ecr20::ABI::approveCall {
                spender: agent.address,
                tokens: start_balance,
            },
            U256::ZERO,
        )
        .unwrap();
    }

    let mut state = AgentState {
        agents: AgentVec::from(agents),
    };

    run(&mut sim, &mut admin_agent, &mut state, 101, n_steps);

    let _agent_data = state.agents.get_records();
}
