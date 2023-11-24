mod sim;
mod types;

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_verbs")]
fn verbs(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<sim::EmptyEnv>()?;
    m.add_class::<sim::ForkEnv>()?;
    Ok(())
}
