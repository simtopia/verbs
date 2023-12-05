use crate::snapshot::PyDbState;

use super::snapshot;
use super::types::{
    address_to_py, event_to_py, result_to_py, PyAddress, PyEvent, PyExecutionResult, PyRevertError,
};
use alloy_primitives::{Address, U256};
use fork_evm::Backend;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use revm::db::{DatabaseRef, EmptyDB};
use rust_sim::contract::Call;
use rust_sim::network::{BlockNumber, Network, RevertError};
use std::mem;

// Represents blocks updating every 15s
const BLOCK_INTERVAL: u32 = 15;

struct BaseEnv<DB: DatabaseRef> {
    // EVM and deployed protocol
    pub network: Network<DB>,
    // Queue of calls submitted from Python
    pub call_queue: Vec<Call>,
    // RNG source
    pub rng: fastrand::Rng,
    // Current step of the simulation
    pub step: usize,
}

impl BaseEnv<EmptyDB> {
    pub fn new(seed: u64, admin_address: &str) -> Self {
        let network = Network::<EmptyDB>::init(admin_address);

        BaseEnv {
            network,
            call_queue: Vec::new(),
            rng: fastrand::Rng::with_seed(seed),
            step: 0,
        }
    }

    pub fn from_snapshot(seed: u64, snapshot: snapshot::PyDbState) -> Self {
        let mut network = Network::<EmptyDB>::init(snapshot.0.as_str());
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
    pub fn new(node_url: &str, seed: u64, block_number: u64, admin_address: &str) -> Self {
        let block_number = match block_number {
            0 => BlockNumber::Latest,
            n => BlockNumber::Number(n.into()),
        };

        let network = Network::<Backend>::init(node_url, block_number, admin_address);
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
            .process_calls(mem::take(&mut self.call_queue), self.step);
        // Clear call queue
        self.step += 1;
    }

    pub fn get_last_events<'a>(&'a self, py: Python<'a>) -> Vec<PyEvent> {
        self.network
            .last_events
            .iter()
            .map(|e| event_to_py(py, e))
            .collect()
    }

    pub fn export_state<'a>(&mut self, py: Python<'a>) -> snapshot::PyDbState<'a> {
        snapshot::create_py_snapshot(py, &mut self.network)
    }

    pub fn submit_call(
        &mut self,
        function_signature: &'static str,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        checked: bool,
    ) {
        self.call_queue.push(Call {
            function_name: function_signature,
            callee: Address::from_slice(&sender),
            transact_to: Address::from_slice(&transact_to),
            args: encoded_args,
            checked,
        })
    }

    pub fn deploy_contract(&mut self, contract_name: &str, bytecode: Vec<u8>) -> Address {
        self.network.deploy_contract(contract_name, bytecode)
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

#[pyclass(unsendable)]
pub struct EmptyEnv(BaseEnv<EmptyDB>);

#[pymethods]
impl EmptyEnv {
    #[new]
    pub fn new(seed: u64, admin_address: &str, snapshot: Option<PyDbState>) -> PyResult<Self> {
        Ok(match snapshot {
            Some(s) => Self(BaseEnv::<EmptyDB>::from_snapshot(seed, s)),
            None => Self(BaseEnv::<EmptyDB>::new(seed, admin_address)),
        })
    }

    pub fn export_snapshot<'a>(&mut self, py: Python<'a>) -> PyResult<snapshot::PyDbState<'a>> {
        Ok(self.0.export_state(py))
    }

    #[getter]
    fn get_step(&self) -> PyResult<usize> {
        Ok(self.0.step)
    }

    #[getter]
    fn get_admin_address<'a>(&self, py: Python<'a>) -> &'a PyBytes {
        address_to_py(py, self.0.network.admin_address)
    }

    pub fn process_block(&mut self) -> PyResult<()> {
        self.0.process_block();
        Ok(())
    }

    pub fn get_last_events<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_last_events(py))
    }

    pub fn submit_call(
        &mut self,
        // function_signature: &'static str,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        checked: bool,
    ) -> PyResult<()> {
        self.0
            .submit_call("fname", sender, transact_to, encoded_args, checked);
        Ok(())
    }

    pub fn deploy_contract<'a>(
        &mut self,
        py: Python<'a>,
        contract_name: &str,
        bytecode: Vec<u8>,
    ) -> PyResult<&'a PyBytes> {
        Ok(PyBytes::new(
            py,
            self.0.deploy_contract(contract_name, bytecode).as_slice(),
        ))
    }

    pub fn create_account(&mut self, address: PyAddress, start_balance: u128) {
        self.0.create_account(address, start_balance)
    }

    pub fn call<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> Result<PyExecutionResult, PyRevertError> {
        let result = self
            .0
            .call(py, sender, contract_address, encoded_args, value);
        match result {
            Ok(x) => Ok(x),
            Err(x) => Err(x.into()),
        }
    }

    pub fn execute<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> Result<PyExecutionResult, PyRevertError> {
        let result = self
            .0
            .execute(py, sender, contract_address, encoded_args, value);
        match result {
            Ok(x) => Ok(x),
            Err(x) => Err(x.into()),
        }
    }
}

#[pyclass(unsendable)]
pub struct ForkEnv(BaseEnv<Backend>);

#[pymethods]
impl ForkEnv {
    #[new]
    pub fn new(
        node_url: &str,
        seed: u64,
        block_number: u64,
        admin_address: &str,
    ) -> PyResult<Self> {
        Ok(Self(BaseEnv::<Backend>::new(
            node_url,
            seed,
            block_number,
            admin_address,
        )))
    }

    pub fn export_snapshot<'a>(&mut self, py: Python<'a>) -> PyResult<snapshot::PyDbState<'a>> {
        Ok(self.0.export_state(py))
    }

    #[getter]
    fn get_step(&self) -> PyResult<usize> {
        Ok(self.0.step)
    }

    #[getter]
    fn get_admin_address<'a>(&self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        Ok(address_to_py(py, self.0.network.admin_address))
    }

    pub fn process_block(&mut self) -> PyResult<()> {
        self.0.process_block();
        Ok(())
    }

    pub fn get_last_events<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_last_events(py))
    }

    pub fn submit_call(
        &mut self,
        // function_signature: &'static str,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        checked: bool,
    ) -> PyResult<()> {
        self.0
            .submit_call("fname", sender, transact_to, encoded_args, checked);
        Ok(())
    }

    pub fn deploy_contract<'a>(
        &mut self,
        py: Python<'a>,
        contract_name: &str,
        bytecode: Vec<u8>,
    ) -> PyResult<&'a PyBytes> {
        Ok(PyBytes::new(
            py,
            self.0.deploy_contract(contract_name, bytecode).as_slice(),
        ))
    }

    pub fn create_account(&mut self, address: PyAddress, start_balance: u128) {
        self.0.create_account(address, start_balance)
    }

    pub fn call<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> Result<PyExecutionResult, PyRevertError> {
        let result = self
            .0
            .call(py, sender, contract_address, encoded_args, value);
        match result {
            Ok(x) => Ok(x),
            Err(x) => Err(x.into()),
        }
    }

    pub fn execute<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> Result<PyExecutionResult, PyRevertError> {
        let result = self
            .0
            .execute(py, sender, contract_address, encoded_args, value);
        match result {
            Ok(x) => Ok(x),
            Err(x) => Err(x.into()),
        }
    }
}
