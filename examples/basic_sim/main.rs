use ethers_core::types::{Address, U256};
use rand::distributions::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use revm::primitives::Address as RevmAddress;
use rust_sim::agent::Agent;
use rust_sim::conract::ContractDefinition;
use rust_sim::network::{ContractCall, SimulationEnvironment};
use rust_sim::sim_runner::SimRunner;

struct SimpleAgent {
    call_address: RevmAddress,
    address: Address,
    dist: Uniform<u64>,
}

impl SimpleAgent {
    fn new(idx: u64, n_agents: u64) -> Self {
        let call_address = RevmAddress::from(idx);
        let address = Address::from(primitive_types::H160::from_low_u64_be(idx));
        let dist = Uniform::from(0..n_agents);
        SimpleAgent {
            call_address,
            address,
            dist,
        }
    }
}

impl Agent for SimpleAgent {
    fn update(&mut self, rng: &mut ThreadRng, network: &mut SimulationEnvironment) {
        let balance_call = ContractCall {
            callee: self.call_address,
            function_name: "balanceOf",
            contract_idx: 0,
            args: (self.address,),
        };
        let current_balance: U256 = network.call_contract(balance_call);

        if current_balance > U256::from(0u64) {
            let receiver = self.dist.sample(rng);
            let receiver = Address::from(primitive_types::H160::from_low_u64_be(receiver));
            let send_amount = std::cmp::min(current_balance, U256::from(1000));
            let send_call = ContractCall {
                callee: self.call_address,
                function_name: "transfer",
                contract_idx: 0,
                args: (receiver, send_amount),
            };
            let _result: bool = network.call_contract(send_call);
        }
    }
}

pub fn main() {
    let args: Vec<String> = std::env::args().collect();

    let n_users: u64 = args[1].parse::<u64>().unwrap();

    let contract_abi_path = String::from("./contracts/basic_erc20_contract/basic_erc20.abi");
    let contract_params_path =
        String::from("./contracts/basic_erc20_contract/basic_erc20_params.json");

    let contract = ContractDefinition::load(contract_abi_path, contract_params_path);

    let start_balance = 1000000000000u128;

    let mut sim = SimulationEnvironment::new(start_balance, n_users);
    sim.deploy_contract(contract);

    let mut agents = Vec::<SimpleAgent>::new();

    for i in 0..n_users {
        let agent = SimpleAgent::new(i, n_users);
        agents.push(agent);
    }

    let start_balance = U256::from(start_balance);
    for agent in &agents {
        let result_call = ContractCall {
            callee: agent.call_address,
            function_name: "approve",
            contract_idx: 0,
            args: (agent.address, start_balance),
        };
        let _result: bool = sim.call_contract(result_call);
    }

    let mut sim_runner: SimRunner<SimpleAgent> = SimRunner::new(sim, agents, 100);
    sim_runner.run();
}
