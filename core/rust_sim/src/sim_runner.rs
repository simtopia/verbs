use crate::agent::SimState;
use crate::network::Env;
use alloy_primitives::U256;
use db::DB;
use kdam::tqdm;

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u32 = 15;

pub fn run<S: SimState, D: DB>(network: &mut Env<D>, agents: &mut S, seed: u64, n_steps: usize) {
    let mut rng = fastrand::Rng::with_seed(seed);

    for i in tqdm!(0..n_steps) {
        // Update all agents
        let mut calls = agents.call_agents(&mut rng, network);
        // Shuffle calls
        rng.shuffle(calls.as_mut_slice());
        // Process calls in order
        network.process_transactions(calls, i);
        // Record data from agents
        agents.record_agents();
        // Move the events from this block into historical storage
        network.clear_events();
        // Update the block-time and number
        network.evm.env.block.timestamp += U256::from(BLOCK_INTERVAL);
        network.evm.env.block.number += U256::from(1);
    }
}
