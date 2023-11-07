use crate::agent::{AdminAgent, SimState};
use crate::network::Network;
use kdam::tqdm;
use revm::db::DatabaseRef;

pub fn run<A: AdminAgent, S: SimState, D: DatabaseRef>(
    network: &mut Network<D>,
    admin_agent: &mut A,
    agents: &mut S,
    seed: u64,
    n_steps: usize,
) {
    let mut rng = fastrand::Rng::with_seed(seed);

    for i in tqdm!(0..n_steps) {
        // Update admin-agent
        admin_agent.update(&mut rng, network);
        // Update all agents
        let mut calls = agents.call_agents(&mut rng, network);
        // Shuffle calls
        rng.shuffle(calls.as_mut_slice());
        // Process calls in order
        network.process_calls(calls, i);
        // Record data from agents
        agents.record_agents();
        // Post block update admin agent
        admin_agent.post_update(network);
        // Move the events from this block into historical storage
        network.clear_events();
    }
}
