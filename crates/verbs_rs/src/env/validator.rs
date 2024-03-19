use rand::{seq::SliceRandom, Rng};

use crate::contract::Transaction;

pub trait Validator {
    fn order_transactions<R: Rng>(
        &mut self,
        rng: &mut R,
        transactions: Vec<Transaction>,
    ) -> Vec<Transaction>;
}

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
