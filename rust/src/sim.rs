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
        let block_env = snapshot::load_block_env(&snapshot);

        let mut network = Network::<EmptyDB>::init(snapshot.0.as_str());
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
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        checked: bool,
    ) {
        self.call_queue.push(Call {
            function_selector: encoded_args[..4].try_into().unwrap(),
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

/// Simulation environment initialised with an empty in-memory database
///
/// Wraps an EVM and in-memory db along with additional functionality
/// for simulation updates and event tracking.
///
#[pyclass]
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

    /// Export a snap shot of the EVM state and block parameters
    ///
    /// Creates a copy of the EVM storage and state of the current block in a
    /// format that can be exported to Python. This snapshot can then be used
    /// to initialise new simulation environments.
    pub fn export_snapshot<'a>(&mut self, py: Python<'a>) -> PyResult<snapshot::PyDbState<'a>> {
        Ok(self.0.export_state(py))
    }

    /// Current step (i.e. block) of the simulation
    #[getter]
    fn get_step(&self) -> PyResult<usize> {
        Ok(self.0.step)
    }

    /// Admin account address
    #[getter]
    fn get_admin_address<'a>(&self, py: Python<'a>) -> &'a PyBytes {
        address_to_py(py, self.0.network.admin_address)
    }

    /// Process the next block in the simulation
    ///
    /// Update the state of the simulation by processing the next
    /// simulated block. This performs several steps:
    ///
    /// * Update the simulated time and block number
    /// * Sort the queue of calls submitted by agents
    /// * Process the queue of calls, updating the state of the EVM
    /// * Store any events generated by the transactions in this block
    ///
    pub fn process_block(&mut self) -> PyResult<()> {
        self.0.process_block();
        Ok(())
    }

    /// Get a list of events/logs generated in the last block
    ///
    /// Returns a list of events/logs generated in the last block.
    /// Events are a tuple containing:
    ///
    /// * The name of the function called
    /// * A vector of logs
    /// * The step the event was generated
    /// * The order the event was created inside a block
    ///
    /// Returns
    /// -------
    /// typing.List[typing.Tuple]
    ///     List of events
    ///
    pub fn get_last_events<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_last_events(py))
    }

    /// Submit a call into the next block
    ///
    /// Submit a transaction into the queue to be processed
    /// in the next block. Each simulation step agents submit
    /// calls which are then shuffled and processed to update
    /// the EVM state.
    ///
    /// Parameters
    /// ----------
    /// sender: bytes
    ///     Byte encoded address of the transaction sender
    /// transact_to: bytes
    ///     Byte encoded address of the contract to call
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments
    /// checked: bool
    ///     If ``True`` the simulation will halt if this transaction
    ///     is reverted
    ///
    pub fn submit_call(
        &mut self,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        checked: bool,
    ) -> PyResult<()> {
        self.0
            .submit_call(sender, transact_to, encoded_args, checked);
        Ok(())
    }

    /// Deploy a contract
    ///
    /// Deploys a contract to the EVM by calling the constructor.
    ///
    /// Parameters
    /// ----------
    /// contract_name: str
    ///     Name of the contract to deploy, only used for
    ///     logging/debugging purposes.
    /// bytecode: bytes
    ///     Contract deployment bytecode and ABI encoded
    ///     constructor arguments.
    ///
    /// Returns
    /// -------
    /// bytes
    ///     Byte encoded address that contract is deployed to.
    ///
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

    /// Create an account
    ///
    /// Create a new account with balance of ETH.
    ///
    /// Parameters
    /// ----------
    /// address: bytes
    ///     Address to deploy account to
    /// start_balance: int
    ///     Starting ETH balance of the account (in wei)
    ///
    pub fn create_account(&mut self, address: PyAddress, start_balance: u128) {
        self.0.create_account(address, start_balance)
    }

    /// Directly call the EVM
    ///
    /// Call the EVM and return the result and events. This
    /// does not update the state of the EVM.
    ///
    /// Parameters
    /// ----------
    /// sender: bytes
    ///     Address of the transaction sender.
    /// contract_address: bytes
    ///     Address of the contract to call.
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments
    ///
    /// Returns
    /// -------
    /// typing.Tuple[typing.Optional[bytes], typing.List[typing.Tuple]]
    ///     Tuple containing optional, byte-encoded results
    ///     of the transaction, and list of logs generated by
    ///     the transaction.
    ///
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

    /// Directly execute a transaction
    ///
    /// Execute a transaction and return the result and events.
    /// This update the state of the EVM.
    ///
    /// Parameters
    /// ----------
    /// sender: bytes
    ///     Address of the transaction sender.
    /// contract_address: bytes
    ///     Address of the contract to call.
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments
    /// value: int
    ///     Value attached to the transaction
    ///
    /// Returns
    /// -------
    /// typing.Tuple[typing.Optional[bytes], typing.List[typing.Tuple]]
    ///     Tuple containing optional, byte-encoded results
    ///     of the transaction, and list of logs generated by
    ///     the transaction.
    ///
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

/// Simulation environment initialised with a fork backend
///
/// Wraps an EVM and in-memory with a fork backend. This allows
/// the EVM to retrieve data from a remote endpoint, to
/// run simulation on forks of actual networks.
///
/// Notes
/// -----
/// Due to requests made by the backend this environment
/// is a lot slower than a purely in memory deployment. One
/// use-case is to run a dummy simulation to retrieve
/// storage values and contracts required for a simulation,
/// then use a snapshot of this environment to initialise
/// in memory simulations.
///
#[pyclass]
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

    /// Export a snap shot of the EVM state and block parameters
    ///
    /// Creates a copy of the EVM storage and state of the current block in a
    /// format that can be exported to Python. This snapshot can then be used
    /// to initialise new simulation environments.
    pub fn export_snapshot<'a>(&mut self, py: Python<'a>) -> PyResult<snapshot::PyDbState<'a>> {
        Ok(self.0.export_state(py))
    }

    /// Current step (i.e. block) of the simulation
    #[getter]
    fn get_step(&self) -> PyResult<usize> {
        Ok(self.0.step)
    }

    /// Admin account address
    #[getter]
    fn get_admin_address<'a>(&self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        Ok(address_to_py(py, self.0.network.admin_address))
    }

    /// Process the next block in the simulation
    ///
    /// Update the state of the simulation by processing the next
    /// simulated block. This performs several steps:
    ///
    /// * Update the simulated time and block number
    /// * Sort the queue of calls submitted by agents
    /// * Process the queue of calls, updating the state of the EVM
    /// * Store any events generated by the transactions in this block
    ///
    pub fn process_block(&mut self) -> PyResult<()> {
        self.0.process_block();
        Ok(())
    }

    /// Get a list of events/logs generated in the last block
    ///
    /// Returns a list of events/logs generated in the last block.
    /// Events are a tuple containing:
    ///
    /// * The name of the function called
    /// * A vector of logs
    /// * The step the event was generated
    /// * The order the event was created inside a block
    ///
    /// Returns
    /// -------
    /// typing.List[typing.Tuple]
    ///     List of events
    ///
    pub fn get_last_events<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_last_events(py))
    }

    /// Submit a call into the next block
    ///
    /// Submit a transaction into the queue to be processed
    /// in the next block. Each simulation step agents submit
    /// calls which are then shuffled and processed to update
    /// the EVM state.
    ///
    /// Parameters
    /// ----------
    /// sender: bytes
    ///     Byte encoded address of the transaction sender
    /// transact_to: bytes
    ///     Byte encoded address of the contract to call
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments
    /// checked: bool
    ///     If ``True`` the simulation will halt if this transaction
    ///     is reverted
    ///
    pub fn submit_call(
        &mut self,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        checked: bool,
    ) -> PyResult<()> {
        self.0
            .submit_call(sender, transact_to, encoded_args, checked);
        Ok(())
    }

    /// Deploy a contract
    ///
    /// Deploys a contract to the EVM by calling the constructor.
    ///
    /// Parameters
    /// ----------
    /// contract_name: str
    ///     Name of the contract to deploy, only used for
    ///     logging/debugging purposes.
    /// bytecode: bytes
    ///     Contract deployment bytecode and ABI encoded
    ///     constructor arguments.
    ///
    /// Returns
    /// -------
    /// bytes
    ///     Byte encoded address that contract is deployed to.
    ///
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

    /// Create an account
    ///
    /// Create a new account with balance of ETH.
    ///
    /// Parameters
    /// ----------
    /// address: bytes
    ///     Address to deploy account to
    /// start_balance: int
    ///     Starting ETH balance of the account (in wei)
    ///
    pub fn create_account(&mut self, address: PyAddress, start_balance: u128) {
        self.0.create_account(address, start_balance)
    }

    /// Directly call the EVM
    ///
    /// Call the EVM and return the result and events. This
    /// does not update the state of the EVM.
    ///
    /// Parameters
    /// ----------
    /// sender: bytes
    ///     Address of the transaction sender.
    /// contract_address: bytes
    ///     Address of the contract to call.
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments
    ///
    /// Returns
    /// -------
    /// typing.Tuple[typing.Optional[bytes], typing.List[typing.Tuple]]
    ///     Tuple containing optional, byte-encoded results
    ///     of the transaction, and list of logs generated by
    ///     the transaction.
    ///
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

    /// Directly execute a transaction
    ///
    /// Execute a transaction and return the result and events.
    /// This update the state of the EVM.
    ///
    /// Parameters
    /// ----------
    /// sender: bytes
    ///     Address of the transaction sender.
    /// contract_address: bytes
    ///     Address of the contract to call.
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments
    /// value: int
    ///     Value attached to the transaction
    ///
    /// Returns
    /// -------
    /// typing.Tuple[typing.Optional[bytes], typing.List[typing.Tuple]]
    ///     Tuple containing optional, byte-encoded results
    ///     of the transaction, and list of logs generated by
    ///     the transaction.
    ///
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
