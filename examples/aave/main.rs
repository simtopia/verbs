mod agents;
mod deployment;
use rust_sim::agent::AgentSet;
use rust_sim::network::Network;
use rust_sim::sim_runner::SimRunner;

const DAI_INDEX: usize = 17;
const USDC_INDEX: usize = 63;
const FAUCET_INDEX: usize = 23;
const POOL_INDEX: usize = 35;

pub fn main() {
    let borrow_agent_set = deployment::initialise_borrow_agents(100);
    let liquidation_agent_set = deployment::initialise_liquidation_agents(2);

    let mut agents: Vec<Box<dyn AgentSet>> = Vec::new();

    agents.push(Box::new(borrow_agent_set));
    agents.push(Box::new(liquidation_agent_set));

    let start_balance = 1000000000000000u128;
    let admin_address = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266";

    let network = Network::from_agents(start_balance, &agents, admin_address);
    let network = deployment::deploy_contracts(network);

    let network =
        deployment::approve_and_mint(network, agents[0].get_addresses(), DAI_INDEX, 100000u128);
    let network = deployment::approve_and_mint(
        network,
        agents[1].get_addresses(),
        USDC_INDEX,
        10000000000000000u128,
    );

    let mut sim_runner: SimRunner = SimRunner::from_agents(network, 100, agents);

    sim_runner.run(0);
}
