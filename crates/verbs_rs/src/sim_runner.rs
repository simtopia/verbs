use crate::agent::SimState;
use crate::env::Env;
use crate::DB;
use alloy_primitives::U256;
use kdam::tqdm;

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u32 = 15;

pub fn run<S: SimState, D: DB>(env: &mut Env<D>, agents: &mut S, seed: u64, n_steps: usize) {
    let mut rng = fastrand::Rng::with_seed(seed);

    for i in tqdm!(0..n_steps) {
        // Move the events from the previous block into historical storage
        env.clear_events();
        // Update all agents
        let mut calls = agents.call_agents(&mut rng, env);
        // Shuffle calls
        rng.shuffle(calls.as_mut_slice());
        // Update the block-time and number
        env.evm.env.block.timestamp += U256::from(BLOCK_INTERVAL);
        env.evm.env.block.number += U256::from(1);
        // Process calls in order
        env.process_transactions(calls, i);
        // Record data from agents
        agents.record_agents();
    }
}
