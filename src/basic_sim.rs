use ethers_core::types::{Address, U256};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use rust_sim::agent::Agent;
use rust_sim::conract::ContractDefinition;
use rust_sim::network::SimulationEnvironment;
use rust_sim::sim_runner::SimRunner;

struct SimpleAgent {
    idx: u64,
    n_agents: u64,
}

impl Agent for SimpleAgent {
    fn update(&mut self, rng: &mut ThreadRng, network: &mut SimulationEnvironment) {
        let current_balance: U256 = network.call_contract(
            self.idx,
            0,
            "balanceOf",
            (Address::from(primitive_types::H160::from_low_u64_be(
                self.idx,
            )),),
        );

        let dist = Uniform::from(0..self.n_agents);

        if current_balance > U256::from(0u64) {
            let receiver = dist.sample(rng);
            let send_amount = std::cmp::min(current_balance, U256::from(1000));
            let _result: bool = network.call_contract(
                self.idx,
                0,
                "transfer",
                (
                    Address::from(primitive_types::H160::from_low_u64_be(receiver)),
                    send_amount,
                ),
            );
        }
    }
}

pub fn basic_sim() {
    let contract_abi_path = String::from("./contracts/basic_erc20_contract/basic_erc20.abi");
    let contract_params_path =
        String::from("./contracts/basic_erc20_contract/basic_erc20_params.json");

    let n_users = 1000;

    let contract = ContractDefinition::load(contract_abi_path, contract_params_path);

    let start_balance: u128 = 1000000000000u128;

    let mut sim = SimulationEnvironment::new(start_balance, n_users);
    sim.deploy_contract(contract);

    for i in 0..n_users {
        let _result: bool = sim.call_contract(
            i,
            0,
            "approve",
            (
                Address::from(primitive_types::H160::from_low_u64_be(i)),
                U256::from(start_balance),
            ),
        );
    }

    let mut agents = Vec::<SimpleAgent>::new();

    for i in 0..n_users {
        let agent = SimpleAgent {
            idx: i,
            n_agents: n_users,
        };
        agents.push(agent);
    }

    let mut sim_runner: SimRunner<SimpleAgent> = SimRunner::new(sim, agents, 100);
    sim_runner.run();
}
