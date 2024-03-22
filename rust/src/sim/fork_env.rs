use super::base_env::BaseEnv;
use super::interface_macro::create_interface;
use super::snapshot;
use crate::types::{PyAddress, PyEvent, PyExecutionResult, PyRevertError, PyTransaction};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use verbs_rs::env::{GasPriorityValidator, RandomValidator};
use verbs_rs::ForkDb;

/// Simulation environment initialised with a fork backend
///
/// Wraps an EVM and in-memory with a fork backend. This allows
/// the EVM to retrieve data from a remote endpoint, to
/// run simulation on forks of actual networks.
///
/// This environment randomly shuffles transactions for inclusion in
/// the next block during a simulation.
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
///    env = verbs.envs.ForkEnvRandom(alchemy_url, 101, block_number=12345)
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
pub struct ForkEnvRandom(BaseEnv<ForkDb, RandomValidator>);

create_interface!(
    ForkEnvRandom,
    #[pyo3(signature = (node_url, seed, block_number = None))]
    pub fn new(node_url: &str, seed: u64, block_number: Option<u64>) -> PyResult<Self> {
        Ok(Self(BaseEnv::<ForkDb, RandomValidator>::new(
            node_url,
            seed,
            block_number,
            RandomValidator {},
        )))
    },
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
    ///    env = verbs.envs.ForkEnv(alchemy_url, 101, block_number=12345)
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
            self.0.env.get_request_history(),
        ))
    }
);

/// Simulation environment initialised with a fork backend
///
/// Wraps an EVM and in-memory with a fork backend. This allows
/// the EVM to retrieve data from a remote endpoint, to
/// run simulation on forks of actual networks.
///
/// This environment sorts transactions by nonce and gas-priority, i.e.
/// each step the queue queue of transactions is:
///
/// - Grouped by transaction sender
/// - Each group sorted by nonce
/// - Groups sorted by the gas-priority fee of the first transaction
/// - The sorted groups are flattened into a vector processing
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
///    env = verbs.envs.ForkEnvGasPriority(alchemy_url, 101, block_number=12345)
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
pub struct ForkEnvGasPriority(BaseEnv<ForkDb, GasPriorityValidator>);

create_interface!(
    ForkEnvGasPriority,
    #[pyo3(signature = (node_url, seed, block_number = None))]
    pub fn new(node_url: &str, seed: u64, block_number: Option<u64>) -> PyResult<Self> {
        Ok(Self(BaseEnv::<ForkDb, GasPriorityValidator>::new(
            node_url,
            seed,
            block_number,
            GasPriorityValidator {},
        )))
    },
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
    ///    env = verbs.envs.ForkEnv(alchemy_url, 101, block_number=12345)
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
            self.0.env.get_request_history(),
        ))
    }
);
