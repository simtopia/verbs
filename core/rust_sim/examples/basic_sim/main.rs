mod ecr20;
mod state;

use alloy_primitives::{Address, Uint, U256};
use rust_sim::agent::AgentVec;
use rust_sim::network::Env;
use rust_sim::sim_runner::run;
use rust_sim::{utils, LocalDB};
use state::{AgentState, SimpleAgent};

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n_users: usize = args[1].parse::<usize>().unwrap();
    let n_steps: usize = args[2].parse::<usize>().unwrap();

    let start_balance = 1000000000000u128;
    let admin_address = Address::from(Uint::from(999));

    let mut sim = Env::<LocalDB>::init(U256::ZERO, U256::ZERO);

    let token_address = sim.deploy_contract(
        admin_address,
        "ECR20",
        utils::constructor_data(ecr20::BYTECODE, None),
    );

    let agents: Vec<SimpleAgent> = (0..n_users)
        .into_iter()
        .map(|x| SimpleAgent::new(x, n_users, token_address))
        .collect();

    sim.insert_accounts(start_balance, agents.iter().map(|a| a.address).collect());

    for agent in &agents {
        sim.direct_execute(
            agent.address,
            token_address,
            ecr20::ABI::approveCall {
                spender: agent.address,
                tokens: U256::from(start_balance),
            },
            U256::ZERO,
        )
        .unwrap();
    }

    let mut state = AgentState {
        agents: AgentVec::from(agents),
    };

    run(&mut sim, &mut state, 101, n_steps);

    let _agent_data = state.agents.get_records();
}
