//! Simulation execution
//!
//! The sim-runner function updates the simulation
//! state (i.e. the agents and EVM) over a fixed
//! number of steps, where each step represents
//! a new block on the simulated chain.
//!

use crate::agent::SimState;
use crate::env::{Env, Validator};
use crate::DB;
use kdam::tqdm;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u64 = 15;

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
pub fn run<S: SimState, D: DB, V: Validator>(
    env: &mut Env<D, V>,
    agents: &mut S,
    seed: u64,
    n_steps: usize,
) {
    let mut rng = Xoroshiro128StarStar::seed_from_u64(seed);

    for i in tqdm!(0..n_steps) {
        // Move the events from the previous block into historical storage
        env.clear_events();
        // Update all agents
        let transactions = agents.call_agents(&mut rng, env);
        // Update the block-time and number
        env.increment_time(&mut rng, BLOCK_INTERVAL);
        // Process calls in order
        env.process_transactions(transactions, &mut rng, i);
        // Record data from agents
        agents.record_agents(env);
    }
}
