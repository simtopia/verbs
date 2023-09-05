use crate::agent::{AdminAgent, AgentSet, AgentSetVec};
use crate::contract::Call;
use crate::network::Network;
use ethers_core::abi::Detokenize;
use kdam::tqdm;

/// Stepped simulation runner
pub struct SimRunner<A: AdminAgent> {
    /// Network/protocol deployment used in the simulation
    pub network: Network,
    /// Admin agent
    pub admin_agent: A,
    /// Collection of agents sets
    pub agents: AgentSetVec,
    /// Current simulation step/block
    pub step: i64,
}

impl<A: AdminAgent> SimRunner<A> {
    /// Initialise a new empty sim-runner.
    ///
    /// # Arguments
    ///
    /// * `network` - Network/deployment.
    /// * `admin_agent` - Simulation admin agent, updated at that start of each step.
    ///
    pub fn new(network: Network, admin_agent: A) -> Self {
        SimRunner {
            network,
            admin_agent,
            agents: AgentSetVec::new(),
            step: 0,
        }
    }

    /// Initialise a sim-runner from  set of agents.
    ///
    /// # Arguments
    ///
    /// * `network` - Network/deployment.
    /// * `admin_agents` - Simulation admin agent, updated at that start of each step.
    /// * `agents` - Collection of simulation agents.
    ///
    pub fn from_agents(network: Network, admin_agent: A, agents: AgentSetVec) -> Self {
        SimRunner {
            network,
            admin_agent,
            agents,
            step: 0,
        }
    }

    /// Insert a set of agents into the simulation
    ///
    /// # Arguments
    ///
    /// * `agent_set` - Reference to a set of agents
    ///
    pub fn insert_agent_set<S: AgentSet + 'static>(&mut self, agent_set: S) {
        self.agents.0.push(Box::new(agent_set));
    }

    /// Run the simulation.
    ///
    /// # Arguments
    ///
    /// * `seed` - Random seed.
    /// * `n_steps` - Number of steps to run the simulation for.
    ///
    pub fn run(&mut self, seed: u64, n_steps: usize) {
        let mut rng = fastrand::Rng::with_seed(seed);

        for _ in tqdm!(0..n_steps) {
            let n = &mut self.network;

            self.admin_agent.update(&mut rng, n);

            let mut calls: Vec<Call> = (&mut self.agents.0)
                .into_iter()
                .map(|x| x.call_agents(&mut rng, n))
                .flatten()
                .collect();

            rng.shuffle(calls.as_mut_slice());
            self.network.process_calls(calls, self.step);
            for agent_set in &mut self.agents.0 {
                agent_set.record_agents();
            }
            self.step += 1;
        }
    }

    /// Decode events of a specific type into actual data
    ///
    /// # Arguments
    ///
    /// * `function_name` - Name of the function that produced the events
    /// * `event_name` - Name of the actual event to decode
    ///
    pub fn process_events<R: Detokenize>(
        &self,
        function_name: &'static str,
        event_name: &'static str,
    ) -> Vec<(i64, R)> {
        self.network
            .events
            .iter()
            .filter(|x| x.function_name == function_name)
            .map(|x| {
                (
                    x.step,
                    self.network.contracts[x.contract_idx]
                        .decode_event(event_name, x.logs.last().unwrap().to_owned()),
                )
            })
            .collect()
    }
}
