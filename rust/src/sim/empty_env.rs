use super::snapshot::PyDbState;

use super::base_env::BaseEnv;
use super::snapshot;
use crate::types::{address_to_py, PyAddress, PyEvent, PyExecutionResult, PyRevertError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use revm::db::EmptyDB;

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
    /// * The selector of the function called
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

    /// Returns a list of events/logs generated over the
    /// course of the simulation.
    /// Events are a tuple containing:
    ///
    /// * The selector of the function called
    /// * A vector of logs
    /// * The step the event was generated
    /// * The order the event was created inside a block
    ///
    /// Returns
    /// -------
    /// typing.List[typing.Tuple]
    ///     List of events
    ///
    pub fn get_event_history<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_event_history(py))
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
    ///     Byte encoded address of the transaction sender.
    /// transact_to: bytes
    ///     Byte encoded address of the contract to call.
    /// encoded_args: bytes
    ///     ABI encoded function selector and arguments.
    /// value: int
    ///     Value attached to the transaction.
    /// checked: bool
    ///     If ``True`` the simulation will halt if this transaction
    ///     is reverted.
    ///
    pub fn submit_transaction(
        &mut self,
        sender: PyAddress,
        transact_to: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
        checked: bool,
    ) -> PyResult<()> {
        self.0
            .submit_transaction(sender, transact_to, encoded_args, value, checked);
        Ok(())
    }

    /// Submit a list of transactions into the next block
    ///
    /// Submit a list of transaction into the queue to be processed
    /// in the next block. Each simulation step agents submit
    /// calls which are then shuffled and processed to update
    /// the EVM state.
    ///
    /// Parameters
    /// ----------
    /// transactions: List[Tuple(bytes, bytes, bytes, int, bool)]
    ///     List of transactions.
    ///
    pub fn submit_transactions(
        &mut self,
        transactions: Vec<(PyAddress, PyAddress, Vec<u8>, u128, bool)>,
    ) -> PyResult<()> {
        self.0.submit_transactions(transactions);
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
