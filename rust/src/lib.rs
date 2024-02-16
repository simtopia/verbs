mod sim;
mod types;

use pyo3::prelude::*;

/// Rust simulation environments
///
/// Python interfaces to rust simulation environments
/// wrapping the EVM and simulation update functionality.
///
#[pymodule]
#[pyo3(name = "envs")]
fn verbs(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<sim::EmptyEnv>()?;
    m.add_class::<sim::ForkEnv>()?;
    m.add("RevertError", py.get_type::<types::PyRevertError>())?;
    Ok(())
}
