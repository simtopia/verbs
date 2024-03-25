use alloy_primitives::{Address, Bytes};
use pyo3::exceptions::PyException;
use pyo3::types::PyBytes;
use pyo3::{create_exception, Python};
use revm::primitives::ExecutionResult;
use std::borrow::Cow;
use verbs_rs::{contract::Event, env::RevertError};

create_exception!(envs, PyRevertError, PyException);

pub type PyAddress<'a> = Cow<'a, [u8]>;

pub type PyTransaction<'a> = (
    PyAddress<'a>,
    PyAddress<'a>,
    Vec<u8>,
    bool,
    Option<u128>,
    Option<u64>,
    Option<u128>,
);

pub fn address_to_py(py: Python, address: Address) -> &PyBytes {
    PyBytes::new(py, address.as_slice())
}

pub fn bytes_to_py(py: Python, bytes: Bytes) -> &PyBytes {
    PyBytes::new(py, bytes.to_vec().as_slice())
}

pub type PyLog<'a> = (&'a PyBytes, &'a PyBytes);

pub type PyEvent<'a> = (bool, &'a PyBytes, Vec<PyLog<'a>>, usize, usize);

pub type PyExecutionResult<'a> = (Option<&'a PyBytes>, Vec<PyLog<'a>>, u64);

pub fn event_to_py<'a>(py: Python<'a>, event: &Event) -> PyEvent<'a> {
    (
        event.success,
        PyBytes::new(py, &event.function_selector),
        event
            .logs
            .iter()
            .map(|x| {
                (
                    address_to_py(py, x.address),
                    PyBytes::new(py, x.data.data.to_vec().as_slice()),
                )
            })
            .collect(),
        event.step,
        event.sequence,
    )
}

pub fn result_to_py(
    py: Python,
    result: Result<ExecutionResult, RevertError>,
) -> Result<PyExecutionResult, RevertError> {
    match result {
        Ok(x) => Ok((
            x.output().map(|b| PyBytes::new(py, b.to_vec().as_slice())),
            x.logs()
                .iter()
                .map(|a| {
                    (
                        address_to_py(py, a.address),
                        bytes_to_py(py, a.data.data.0.clone().into()),
                    )
                })
                .collect(),
            x.gas_used(),
        )),
        Err(x) => Err(x.clone()),
    }
}
