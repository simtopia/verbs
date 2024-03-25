//! Validator trait and implementations
//!
use alloy_primitives::{Address, U256};
use rand::{seq::SliceRandom, Rng};

use crate::contract::Transaction;
use std::collections::HashMap;

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

/// Validator that sorts transactions by Nonce and priority fee
///
/// This implementation is adapted from the geth implementation:
///
/// > This method first sorts the separates the list of transactions into individual
/// sender accounts and sorts them by nonce. After the account nonce ordering is
/// satisfied, the results are merged back together by price, always comparing only
/// the head transaction from each account.
///
/// As such it performs the following steps:
///
/// - Group transactions by sender address
/// - Sort individual groups by nonce
/// - Sort the groups by the gas-priority of the first transaction in the group
/// - Flatten the groups into a single vector for processing
///
pub struct GasPriorityValidator {}

impl Validator for GasPriorityValidator {
    fn order_transactions<R: Rng>(
        &mut self,
        _rng: &mut R,
        transactions: Vec<Transaction>,
    ) -> Vec<Transaction> {
        let mut transaction_by_address = HashMap::<Address, Vec<Transaction>>::new();

        for t in transactions.into_iter() {
            match transaction_by_address.entry(t.callee) {
                std::collections::hash_map::Entry::Occupied(mut o) => {
                    o.get_mut().push(t);
                }
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(vec![t]);
                }
            };
        }

        let mut transactions: Vec<Vec<Transaction>> = transaction_by_address
            .into_values()
            .map(|mut v| {
                v.sort_by_key(|x| x.nonce);
                v
            })
            .collect();

        transactions.sort_by_key(|x| U256::MAX - x[0].gas_priority_fee.unwrap_or(U256::ZERO));

        transactions.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_primitives::{Address, Uint, U256};
    use rand::SeedableRng;
    use rand_xoshiro::Xoroshiro128StarStar;

    #[test]
    fn test_gas_priority() {
        let mut rng = Xoroshiro128StarStar::seed_from_u64(101);

        let address_a = Address::from(Uint::from(101u128));
        let address_b = Address::from(Uint::from(202u128));

        let transactions = vec![
            Transaction {
                function_selector: [0; 4],
                callee: address_a,
                transact_to: Address::ZERO,
                args: Vec::default(),
                gas_priority_fee: Some(U256::from(10)),
                nonce: Some(1),
                value: U256::ZERO,
                checked: false,
            },
            Transaction {
                function_selector: [0; 4],
                callee: address_b,
                transact_to: Address::ZERO,
                args: Vec::default(),
                gas_priority_fee: None,
                nonce: Some(1),
                value: U256::ZERO,
                checked: false,
            },
            Transaction {
                function_selector: [0; 4],
                callee: address_a,
                transact_to: Address::ZERO,
                args: Vec::default(),
                gas_priority_fee: None,
                nonce: Some(2),
                value: U256::ZERO,
                checked: false,
            },
            Transaction {
                function_selector: [0; 4],
                callee: address_b,
                transact_to: Address::ZERO,
                args: Vec::default(),
                gas_priority_fee: None,
                nonce: Some(2),
                value: U256::ZERO,
                checked: false,
            },
        ];

        let mut validator = GasPriorityValidator {};

        let transactions = validator.order_transactions(&mut rng, transactions);

        assert!(transactions.len() == 4);

        assert!(transactions[0].callee == address_a);
        assert!(transactions[0].nonce == Some(1));

        assert!(transactions[1].callee == address_a);
        assert!(transactions[1].nonce == Some(2));

        assert!(transactions[2].callee == address_b);
        assert!(transactions[2].nonce == Some(1));

        assert!(transactions[3].callee == address_b);
        assert!(transactions[3].nonce == Some(2));
    }
}
