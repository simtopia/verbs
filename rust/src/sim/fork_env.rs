use super::base_env::BaseEnv;
use super::snapshot;
use crate::types::{PyAddress, PyEvent, PyExecutionResult, PyRevertError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use verbs_rs::ForkDb;

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
/// use-case is to run a simulation to retrieve
/// storage values and contracts required for a simulation,
/// then use the cache from this environment to initialise
/// other in memory simulations.
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///    # Here alchemy_url is url string to the alchemy API
///    env = verbs.envs.ForkEnv(alchemy_url, 101, 12345)
///    ...
///    env.submit_call(...)
///
/// To then use the cache from this simulation to
/// run subsequent simulations
///
/// .. code-block:: python
///
///    cache = env.export_cache()
///
///    new_env = verbs.envs.EmptyEnv(101, cache=cache)
///
#[pyclass]
pub struct ForkEnv(BaseEnv<ForkDb>);

#[pymethods]
impl ForkEnv {
    #[new]
    pub fn new(node_url: &str, seed: u64, block_number: u64) -> PyResult<Self> {
        Ok(Self(BaseEnv::<ForkDb>::new(node_url, seed, block_number)))
    }

    /// Export a snap shot of the EVM state and block parameters
    ///
    /// Creates a copy of the EVM storage and state of the current block in a
    /// format that can be exported to Python. This snapshot can then be used
    /// to initialise new simulation environments.
    pub fn export_snapshot<'a>(&mut self, py: Python<'a>) -> PyResult<snapshot::PyDbState<'a>> {
        Ok(self.0.export_state(py))
    }

    /// Export a cache of calls made by the DB
    ///
    /// Exports a cache of requests made to the remote endpoint, i.e. requests
    /// for account data and storage values. This data can then be used
    /// to initialise the db for a purely local database for use in other
    /// simulations.
    ///
    /// Examples
    /// --------
    ///
    /// .. code-block:: python
    ///
    ///    env = verbs.envs.ForkEnv(alchemy_url, 101, 12345)
    ///    # Run simulation from fork
    ///    ...
    ///    # Get cache of requests
    ///    cache = env.export_cache()
    ///    # Directly initialise a new environment from the cache
    ///    new_env = verbs.envs.EmptyEnv(101, cache=cache)
    ///
    /// Returns
    /// -------
    ///
    /// list[tuple]
    ///     Tuple containing:
    ///
    ///     - Env block timestamp
    ///     - Env block number
    ///     - List of account info requests
    ///     - List of storage value requests
    ///
    pub fn export_cache<'a>(&mut self, py: Python<'a>) -> PyResult<snapshot::PyRequests<'a>> {
        Ok(snapshot::create_py_request_history(
            py,
            self.0.network.get_request_history(),
        ))
    }

    /// Current step (i.e. block) of the simulation
    ///
    /// Returns
    /// -------
    /// int
    ///     Current step of the simulation.
    ///
    #[getter]
    fn get_step(&self) -> PyResult<usize> {
        Ok(self.0.step)
    }

    /// Process the next block in the simulation
    ///
    /// Update the state of the simulation by processing the next
    /// simulated block. This performs several steps:
    ///
    /// * Update the simulated time and block number
    /// * Randomly shuffle the queue of calls submitted by agents
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
    /// list[tuple]
    ///     List of events
    ///
    pub fn get_last_events<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_last_events(py))
    }

    /// get_event_history() -> list[tuple]
    ///
    ///Returns a list of events/logs generated over the
    /// course of the simulation.
    /// Events are a tuple containing:
    ///
    /// * Boolean indicating if the transaction was successful
    /// * The selector of the function called
    /// * A vector of logs
    /// * The step the event was generated
    /// * The order the event was created inside a block
    ///
    /// Returns
    /// -------
    /// list[tuple]
    ///     List of events
    ///
    pub fn get_event_history<'a>(&'a mut self, py: Python<'a>) -> PyResult<Vec<PyEvent>> {
        Ok(self.0.get_event_history(py))
    }

    /// submit_call(sender: bytes, transact_to: bytes, encoded_args: bytes, value: int, checked: bool)
    ///
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
    pub fn submit_call(
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

    /// submit_transactions(transactions: list[tuple[bytes, bytes, bytes, int, bool]])
    ///
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
    ///     List of transactions, where a transaction is a tuple
    ///     containing:
    ///
    ///     * The byte encoded address of the sender
    ///     * The byte encoded address of the contract
    ///     * The ABI byte encoded arguments and function selector
    ///     * The value attached to the transaction
    ///     * Flag if ``True`` means the simulation will halt if this
    ///       transaction fails
    ///
    pub fn submit_transactions(
        &mut self,
        transactions: Vec<(PyAddress, PyAddress, Vec<u8>, u128, bool)>,
    ) -> PyResult<()> {
        self.0.submit_transactions(transactions);
        Ok(())
    }

    /// deploy_contract(deployer: bytes, contract_name: str, bytecode: bytes) -> bytes
    ///
    /// Deploy a contract
    ///
    /// Deploys a contract to the EVM by calling the constructor.
    ///
    /// Parameters
    /// ----------
    /// deployer: bytes
    ///     Byte encoded address of the deployer.
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
        deployer: PyAddress,
        contract_name: &str,
        bytecode: Vec<u8>,
    ) -> PyResult<&'a PyBytes> {
        Ok(PyBytes::new(
            py,
            self.0
                .deploy_contract(deployer, contract_name, bytecode)
                .as_slice(),
        ))
    }

    /// create_account(address: bytes, start_balance: int)
    ///
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

    /// call(sender: bytes, contract_address: bytes, encoded_args: bytes, value: int) -> tuple[bytes, list, int]
    ///
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
    /// value: int
    ///     Value attached to this transaction.
    ///
    /// Returns
    /// -------
    /// tuple[bytes, list[tuple], int]
    ///     Tuple containing optional, byte-encoded results
    ///     of the transaction, and list of logs generated by
    ///     the transaction.
    ///
    /// Raises
    /// ------
    /// verbs.envs.RevertError
    ///     Raises an exception if the transaction is reverted.
    ///
    pub fn call<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> PyResult<PyExecutionResult> {
        let result = self
            .0
            .call(py, sender, contract_address, encoded_args, value);
        match result {
            Ok(x) => Ok(x),
            Err(x) => Err(PyRevertError::new_err(x.output)),
        }
    }

    /// execute(sender: bytes, contract_address: bytes, encoded_args: bytes, value: int) -> tuple[bytes, list, int]
    ///
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
    /// Raises
    /// ------
    /// verbs.envs.RevertError
    ///     Raises an exception if the transaction is reverted.
    ///
    pub fn execute<'a>(
        &'a mut self,
        py: Python<'a>,
        sender: PyAddress,
        contract_address: PyAddress,
        encoded_args: Vec<u8>,
        value: u128,
    ) -> PyResult<PyExecutionResult> {
        let result = self
            .0
            .execute(py, sender, contract_address, encoded_args, value);
        match result {
            Ok(x) => Ok(x),
            Err(x) => Err(PyRevertError::new_err(x.output)),
        }
    }
}
