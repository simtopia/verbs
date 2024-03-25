use super::snapshot::PyDbState;

use super::base_env::BaseEnv;
use super::interface_macro::create_interface;
use super::snapshot;
use crate::types::{PyAddress, PyEvent, PyExecutionResult, PyRevertError, PyTransaction};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use verbs_rs::env::{GasPriorityValidator, RandomValidator};
use verbs_rs::LocalDB;

/// Simulation environment initialised with an empty in-memory database
///
/// Wraps an EVM and in-memory db along with additional functionality
/// for simulation updates and event tracking. This environment can
/// also be initialised from a snapshot to speed up simulation
/// initialisation.
///
/// This environment randomly shuffles transactions for inclusion in
/// the next block during a simulation.
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///    # Initialise a completely empty db
///    env = EmptyEnvRandom(101)
///    # Or initialise from a snapshot
///    env = EmptyEnvRandom(101, snapshot=snapshot)
///    # Or load a cache from a previous forked run
///    env = EmptyEnvRandom(101, cache=cache)
///    ...
///    env.submit_call(...)
///
#[pyclass]
pub struct EmptyEnvRandom(BaseEnv<LocalDB, RandomValidator>);

create_interface!(
    EmptyEnvRandom,
    #[pyo3(signature = (seed, snapshot=None, cache=None))]
    pub fn new(
        seed: u64,
        snapshot: Option<PyDbState>,
        cache: Option<snapshot::PyRequests>,
    ) -> PyResult<Self> {
        let validator = RandomValidator {};

        match (snapshot, cache) {
            (None, None) => Ok(Self(BaseEnv::<LocalDB, RandomValidator>::new(
                0, 0, seed, validator,
            ))),
            (None, Some(c)) => Ok(Self(BaseEnv::<LocalDB, RandomValidator>::from_cache(
                seed, c, validator,
            ))),
            (Some(s), None) => Ok(Self(BaseEnv::<LocalDB, RandomValidator>::from_snapshot(
                seed, s, validator,
            ))),
            (Some(_), Some(_)) => Err(PyRuntimeError::new_err(
                "Env must be initialised from either a snapshot or a cache but not both.",
            )),
        }
    },
    fn dummy(&self) {}
);

/// Simulation environment initialised with an empty in-memory database
///
/// Wraps an EVM and in-memory db along with additional functionality
/// for simulation updates and event tracking. This environment can
/// also be initialised from a snapshot to speed up simulation
/// initialisation.
///
/// This environment sorts transactions by nonce and gas-priority, i.e.
/// each step the queue queue of transactions is:
///
/// - Grouped by transaction sender
/// - Each group sorted by nonce
/// - Groups sorted by the gas-priority fee of the first transaction
/// - The sorted groups are flattened into a vector processing
///
/// Examples
/// --------
///
/// .. code-block:: python
///
///    # Initialise a completely empty db
///    env = EmptyEnvGasPriority(101)
///    # Or initialise from a snapshot
///    env = EmptyEnvGasPriority(101, snapshot=snapshot)
///    # Or load a cache from a previous forked run
///    env = EmptyEnvGasPriority(101, cache=cache)
///    ...
///    env.submit_call(...)
///
#[pyclass]
pub struct EmptyEnvGasPriority(BaseEnv<LocalDB, GasPriorityValidator>);

create_interface!(
    EmptyEnvGasPriority,
    #[pyo3(signature = (seed, snapshot=None, cache=None))]
    pub fn new(
        seed: u64,
        snapshot: Option<PyDbState>,
        cache: Option<snapshot::PyRequests>,
    ) -> PyResult<Self> {
        let validator = GasPriorityValidator {};

        match (snapshot, cache) {
            (None, None) => Ok(Self(BaseEnv::<LocalDB, GasPriorityValidator>::new(
                0, 0, seed, validator,
            ))),
            (None, Some(c)) => Ok(Self(BaseEnv::<LocalDB, GasPriorityValidator>::from_cache(
                seed, c, validator,
            ))),
            (Some(s), None) => Ok(Self(
                BaseEnv::<LocalDB, GasPriorityValidator>::from_snapshot(seed, s, validator),
            )),
            (Some(_), Some(_)) => Err(PyRuntimeError::new_err(
                "Env must be initialised from either a snapshot or a cache but not both.",
            )),
        }
    },
    fn dummy(&self) {}
);
