mod agents;
mod calls;
mod deployment;
mod protocol_deployment;
use rust_sim::agent::AgentSet;
use rust_sim::network::Network;
use rust_sim::sim_runner::SimRunner;

const FAUCET_INDEX: usize = 12;
const TOKEN_A_IDX: usize = 13;
const TOKEN_B_IDX: usize = 14;
const DATA_PROVIDER_INDEX: usize = 22;
const POOL_INDEX: usize = 28;
const ORACLE_INDEX: usize = 33;

pub fn main() {
    let start_balance = 100000000000000000000u128;
    let admin_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let network = Network::init(admin_address);
    let mut network = protocol_deployment::deploy_contracts(network);

    let _token_a_config = calls::get_reserve_configuration_data(&mut network, TOKEN_A_IDX);

    let token_b_config = calls::get_reserve_configuration_data(&mut network, TOKEN_B_IDX);
    let token_b_decimals = token_b_config.0;
    let token_b_ltv = token_b_config.1;

    let token_a_address = network.get_contract_address(TOKEN_A_IDX);
    let token_b_address = network.get_contract_address(TOKEN_B_IDX);

    let borrow_agent_set = deployment::initialise_borrow_agents(
        100,
        0.01,
        token_b_ltv,
        token_b_decimals,
        token_a_address,
        token_b_address,
    );
    let liquidation_agent_set = deployment::initialise_liquidation_agents(
        2,
        token_b_address,
        token_a_address,
        borrow_agent_set.get_addresses(),
    );

    let mut agents: Vec<Box<dyn AgentSet>> = Vec::new();

    agents.push(Box::new(borrow_agent_set));
    agents.push(Box::new(liquidation_agent_set));

    network.insert_agents(start_balance, &agents);

    let network = deployment::approve_and_mint(
        network,
        agents[0].get_addresses(),
        TOKEN_A_IDX,
        100000000000u128,
    );

    let network = deployment::approve_and_mint(
        network,
        agents[1].get_addresses(),
        TOKEN_B_IDX,
        10000000000000000u128,
    );

    let network =
        deployment::admin_mint_and_supply(network, TOKEN_B_IDX, 10000000000000000000000000000000);

    let mut sim_runner: SimRunner = SimRunner::from_agents(network, 100, agents);

    sim_runner.run(0);
}
