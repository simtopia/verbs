use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "_verbs")]
fn simulation_deployment(_py: Python<'_>, _m: &PyModule) -> PyResult<()> {
    Ok(())
}
