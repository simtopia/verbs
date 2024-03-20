//! Validator trait and implementations
//!
use rand::{seq::SliceRandom, Rng};

use crate::contract::Transaction;

/// Trait for a block validator
///
/// Validators are (currently) responsible
/// for the ordering of transactions in the
/// next block during a simulation.
pub trait Validator {
    /// Sort transaction queue
    ///
    /// Sort a vector of transactions for processing
    /// in the next simulated block.
    ///
    /// # Arguments
    ///
    /// - `rng` - Random generator.
    /// - `transactions` Vector submitted transactions.
    ///
    fn order_transactions<R: Rng>(
        &mut self,
        rng: &mut R,
        transactions: Vec<Transaction>,
    ) -> Vec<Transaction>;
}

/// Validator that randomly shuffles transactions
pub struct RandomValidator {}

impl Validator for RandomValidator {
    fn order_transactions<R: Rng>(
        &mut self,
        rng: &mut R,
        mut transactions: Vec<Transaction>,
    ) -> Vec<Transaction> {
        transactions.as_mut_slice().shuffle(rng);
        transactions
    }
}

pub struct GasPriorityValidator {}

impl Validator for GasPriorityValidator {
    fn order_transactions<R: Rng>(
        &mut self,
        rng: &mut R,
        mut transactions: Vec<Transaction>,
    ) -> Vec<Transaction> {
        // Shuffle first to remove any dependency on agent ordering
        transactions.as_mut_slice().shuffle(rng);

        transactions
    }
}
