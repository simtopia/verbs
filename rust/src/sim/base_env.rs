use super::snapshot;
use crate::types::{event_to_py, result_to_py, PyAddress, PyEvent, PyExecutionResult};
use alloy_primitives::{Address, U256};
use fork_evm::Backend;
use pyo3::prelude::*;

use revm::db::{DatabaseRef, EmptyDB};
use rust_sim::contract::Transaction;
use rust_sim::network::{BlockNumber, Network, RevertError};
use std::mem;

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u32 = 15;

pub struct BaseEnv<DB: DatabaseRef> {
    // EVM and deployed protocol
    pub network: Network<DB>,
    // Queue of calls submitted from Python
    pub call_queue: Vec<Transaction>,
    // RNG source
    pub rng: fastrand::Rng,
    // Current step of the simulation
    pub step: usize,
}

impl BaseEnv<EmptyDB> {
    pub fn new(seed: u64) -> Self {
        let network = Network::<EmptyDB>::init();

        BaseEnv {
            network,
            call_queue: Vec::new(),
            rng: fastrand::Rng::with_seed(seed),
            step: 0,
        }
    }

    pub fn from_snapshot(seed: u64, snapshot: snapshot::PyDbState) -> Self {
        let block_env = snapshot::load_block_env(&snapshot);

        let mut network = Network::<EmptyDB>::init();
        network.evm.env.block = block_env;

        snapshot::load_snapshot(network.evm.db().unwrap(), snapshot);
        BaseEnv {
            network,
            call_queue: Vec::new(),
            rng: fastrand::Rng::with_seed(seed),
            step: 0,
        }
    }
}

impl BaseEnv<Backend> {
    pub fn new(node_url: &str, seed: u64, block_number: u64) -> Self {
        let block_number = match block_number {
            0 => BlockNumber::Latest,
            n => BlockNumber::Number(n.into()),
        };

        let network = Network::<Backend>::init(node_url, block_number);
        BaseEnv {
            network,
            call_queue: Vec::new(),
            rng: fastrand::Rng::with_seed(seed),
            step: 0,
        }
    }
}

impl<DB: DatabaseRef> BaseEnv<DB> {
    pub fn process_block(&mut self) {
        // Update the block-time and number
        self.network.evm.env.block.timestamp += U256::from(BLOCK_INTERVAL);
        self.network.evm.env.block.number += U256::from(1);
        // Clear events from last block
        self.network.clear_events();
        // Shuffle and process calls
        self.rng.shuffle(self.call_queue.as_mut_slice());
        self.network
            .process_transactions(mem::take(&mut self.call_queue), self.step);
        // Tick step
        self.step += 1;
    }

    pub fn get_last_events<'a>(&'a self, py: Python<'a>) -> Vec<PyEvent> {
        self.network
            .last_events
            .iter()
            .map(|e| event_to_py(py, e))
            .collect()
    }

    pub fn get_event_history<'a>(&'a self, py: Python<'a>) -> Vec<PyEvent> {
        self.network
            .event_history
            .iter()
            .map(|e| event_to_py(py, e))
            .collect()
    }

    pub fn export_state<'a>(&mut self, py: Python<'a>) -> snapshot::PyDbState<'a> {
        snapshot::create_py_snapshot(py, &mut self.network)
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
        self.network
            .deploy_contract(Address::from_slice(&deployer), contract_name, bytecode)
    }

    pub fn create_account(&mut self, address: PyAddress, start_balance: u128) {
        self.network
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
        let result = self.network.direct_call_raw(
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
        let result = self.network.direct_execute_raw(
            Address::from_slice(&sender),
            Address::from_slice(&contract_address),
            encoded_args,
            value,
        );
        result_to_py(py, result)
    }
}
