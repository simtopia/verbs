mod ecr20;
mod state;

use alloy_primitives::{Address, Uint, U256};
use state::{AgentState, SimpleAgent};
use verbs_rs::agent::AgentVec;
use verbs_rs::env::{Env, RandomValidator};
use verbs_rs::sim_runner::run;
use verbs_rs::{utils, LocalDB};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Number of agents
    #[arg(short, long, default_value_t = 100)]
    n_agents: usize,

    /// Number of simulation steps
    #[arg(short, long, default_value_t = 100)]
    steps: usize,
}

pub fn main() {
    let args = Args::parse();

    let start_balance = 1000000000000u128;
    let admin_address = Address::from(Uint::from(999));

    let mut env = Env::<LocalDB, RandomValidator>::init(U256::ZERO, U256::ZERO, RandomValidator {});

    let token_address = env.deploy_contract(
        admin_address,
        "ECR20",
        utils::constructor_data(ecr20::BYTECODE, None),
    );

    let agents: Vec<SimpleAgent> = (0..args.n_agents)
        .into_iter()
        .map(|x| SimpleAgent::new(x, args.n_agents, token_address))
        .collect();

    env.insert_accounts(start_balance, agents.iter().map(|a| a.address).collect());

    for agent in &agents {
        env.direct_execute(
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

    run(&mut env, &mut state, 101, args.steps);

    let _agent_data = state.agents.get_records();
}
