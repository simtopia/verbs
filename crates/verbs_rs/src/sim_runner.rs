//! Simulation execution
//!
//! The sim-runner function updates the simulation
//! state (i.e. the agents and EVM) over a fixed
//! number of steps, where each step represents
//! a new block on the simulated chain.
//!

use crate::agent::SimState;
use crate::env::Env;
use crate::DB;
use alloy_primitives::U256;
use kdam::tqdm;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u32 = 15;

/// Simulation execution function
///
/// Run a simulation for a fixed number of steps,
/// each step of the simulation:
///
/// * Updates the state of all the agents and
///   collects transactions to be submitted into
///   the next block
/// * Sort the transactions
/// * Update the block number and timestamp
/// * Process the transactions
/// * Record the state of the agents
///
/// # Arguments
///
/// * `env` - Reference to an [Env] simulation environment
/// * `agents` - Reference to a set of agents implementing
///   the [SimState] trait
/// * `seed` - Random seed
/// * `n_steps` - Number of simulation steps
///
pub fn run<S: SimState, D: DB>(env: &mut Env<D>, agents: &mut S, seed: u64, n_steps: usize) {
    let mut rng = Xoroshiro128StarStar::seed_from_u64(seed);

    for i in tqdm!(0..n_steps) {
        // Move the events from the previous block into historical storage
        env.clear_events();
        // Update all agents
        let mut transactions = agents.call_agents(&mut rng, env);
        // Shuffle calls
        transactions.as_mut_slice().shuffle(&mut rng);
        // Update the block-time and number
        env.evm_state
            .as_mut()
            .unwrap()
            .context
            .evm
            .env
            .block
            .timestamp += U256::from(BLOCK_INTERVAL);
        env.evm_state.as_mut().unwrap().context.evm.env.block.number += U256::from(1);
        // Process calls in order
        env.process_transactions(transactions, i);
        // Record data from agents
        agents.record_agents(env);
    }
}
