use super::snapshot::{
    create_py_snapshot, load_block_env, load_cache, load_snapshot, PyDbState, PyRequests,
};
use crate::types::{event_to_py, result_to_py, PyAddress, PyEvent, PyExecutionResult};
use alloy_primitives::{Address, U256};
use pyo3::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_xoshiro::Xoroshiro128StarStar;

use std::mem;
use verbs_rs::contract::Transaction;
use verbs_rs::env::{Env, RevertError};
use verbs_rs::{ForkDb, LocalDB, DB};

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u64 = 15;

pub struct BaseEnv<D: DB> {
    // EVM and deployed protocol
    pub env: Env<D>,
    // Queue of calls submitted from Python
    pub call_queue: Vec<Transaction>,
    // RNG source
    pub rng: Xoroshiro128StarStar,
    // Current step of the simulation
    pub step: usize,
}

impl BaseEnv<LocalDB> {
    pub fn new(timestamp: u128, block_number: u128, seed: u64) -> Self {
        let env = Env::<LocalDB>::init(
            U256::try_from(timestamp).unwrap(),
            U256::try_from(block_number).unwrap(),
        );

        BaseEnv {
            env,
            call_queue: Vec::new(),
            rng: Xoroshiro128StarStar::seed_from_u64(seed),
            step: 0,
        }
    }

    pub fn from_snapshot(seed: u64, snapshot: PyDbState) -> Self {
        let block_env = load_block_env(&snapshot);

        let mut env = Env::<LocalDB>::init(block_env.timestamp, block_env.number);
        env.evm_state().context.evm.env.block = block_env;

        load_snapshot(&mut env.evm_state().context.evm.db, snapshot);

        BaseEnv {
            env,
            call_queue: Vec::new(),
            rng: Xoroshiro128StarStar::seed_from_u64(seed),
            step: 0,
        }
    }

    pub fn from_cache(seed: u64, requests: PyRequests) -> Self {
        let mut env = Env::<LocalDB>::init(
            U256::try_from(requests.0).unwrap(),
            U256::try_from(requests.1).unwrap(),
        );
        load_cache(requests, &mut env.evm_state().context.evm.db);

        BaseEnv {
            env,
            call_queue: Vec::new(),
            rng: Xoroshiro128StarStar::seed_from_u64(seed),
            step: 0,
        }
    }
}

impl BaseEnv<ForkDb> {
    pub fn new(node_url: &str, seed: u64, block_number: Option<u64>) -> Self {
        let env = Env::<ForkDb>::init(node_url, block_number);
        BaseEnv {
            env,
            call_queue: Vec::new(),
            rng: Xoroshiro128StarStar::seed_from_u64(seed),
            step: 0,
        }
    }
}

impl<D: DB> BaseEnv<D> {
    pub fn process_block(&mut self) {
        // Update the block-time and number
        self.env.increment_time(BLOCK_INTERVAL);
        // Clear events from last block
        self.env.clear_events();
        // Shuffle and process calls
        self.call_queue.as_mut_slice().shuffle(&mut self.rng);
        self.env
            .process_transactions(mem::take(&mut self.call_queue), self.step);
        // Tick step
        self.step += 1;
    }

    pub fn get_last_events<'a>(&'a self, py: Python<'a>) -> Vec<PyEvent> {
        self.env
            .last_events
            .iter()
            .map(|e| event_to_py(py, e))
            .collect()
    }

    pub fn get_event_history<'a>(&'a self, py: Python<'a>) -> Vec<PyEvent> {
        self.env
            .event_history
            .iter()
            .map(|e| event_to_py(py, e))
            .collect()
    }

    pub fn export_state<'a>(&mut self, py: Python<'a>) -> PyDbState<'a> {
        create_py_snapshot(py, &self.env)
    }

    pub fn submit_transaction(
        &mut self,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
        checked: bool,
    ) {
        self.call_queue.push(Transaction {
            function_selector: encoded_args[..4].try_into().unwrap(),
            callee: Address::from_slice(&sender),
            transact_to: Address::from_slice(&transact_to),
            args: encoded_args,
            value: U256::try_from(value).unwrap(),
            checked,
        })
    }

    pub fn submit_transactions(
        &mut self,
        transactions: Vec<(PyAddress, PyAddress, Vec<u8>, u128, bool)>,
    ) {
        self.call_queue
            .extend(transactions.into_iter().map(|x| Transaction {
                function_selector: x.2[..4].try_into().unwrap(),
                callee: Address::from_slice(&x.0),
                transact_to: Address::from_slice(&x.1),
                args: x.2,
                value: U256::try_from(x.3).unwrap(),
                checked: x.4,
            }))
    }

    pub fn deploy_contract(
        &mut self,
        deployer: PyAddress,
        contract_name: &str,
        bytecode: Vec<u8>,
    ) -> Address {
        self.env
            .deploy_contract(Address::from_slice(&deployer), contract_name, bytecode)
    }

    pub fn create_account(&mut self, address: PyAddress, start_balance: u128) {
        self.env
            .insert_account(Address::from_slice(&address), U256::from(start_balance))
    }

    pub fn call<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> Result<PyExecutionResult, RevertError> {
        let value = U256::try_from(value).unwrap();
        let result = self.env.direct_call_raw(
            Address::from_slice(&sender),
            Address::from_slice(&contract_address),
            encoded_args,
            value,
        );
        result_to_py(py, result)
    }

    pub fn execute<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> Result<PyExecutionResult, RevertError> {
        let value = U256::try_from(value).unwrap();
        let result = self.env.direct_execute_raw(
            Address::from_slice(&sender),
            Address::from_slice(&contract_address),
            encoded_args,
            value,
        );
        result_to_py(py, result)
    }
}
