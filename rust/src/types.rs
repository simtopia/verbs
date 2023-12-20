use alloy_primitives::{Address, Bytes};
use pyo3::exceptions::PyException;
use pyo3::types::PyBytes;
use pyo3::{PyErr, Python};
use revm::primitives::ExecutionResult;
use rust_sim::{contract::Event, network::RevertError};
use std::{borrow::Cow, fmt};

pub type PyAddress<'a> = Cow<'a, [u8]>;

pub fn address_to_py(py: Python, address: Address) -> &PyBytes {
    PyBytes::new(py, address.as_slice())
}

pub fn bytes_to_py(py: Python, bytes: Bytes) -> &PyBytes {
    PyBytes::new(py, bytes.to_vec().as_slice())
}

pub type PyLog<'a> = (&'a PyBytes, &'a PyBytes);

pub type PyEvent<'a> = (&'a PyBytes, Vec<PyLog<'a>>, usize, usize);

pub type PyExecutionResult<'a> = (Option<&'a PyBytes>, Vec<PyLog<'a>>, u64);

pub fn event_to_py<'a>(py: Python<'a>, event: &Event) -> PyEvent<'a> {
    (
        PyBytes::new(py, &event.function_selector),
        event
            .logs
            .iter()
            .map(|x| {
                (
                    address_to_py(py, x.address),
                    PyBytes::new(py, x.data.to_vec().as_slice()),
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
                .into_iter()
                .map(|a| (address_to_py(py, a.address), bytes_to_py(py, a.data)))
                .collect(),
            x.gas_used(),
        )),
        Err(x) => Err(x.clone()),
    }
}

pub struct PyRevertError {
    reason: Option<String>,
}

impl fmt::Display for PyRevertError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reason = match &self.reason {
            Some(x) => x.as_str(),
            None => "No output",
        };
        write!(f, "{}", reason)
    }
}

impl std::convert::From<PyRevertError> for PyErr {
    fn from(err: PyRevertError) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

impl std::convert::From<RevertError> for PyRevertError {
    fn from(err: RevertError) -> PyRevertError {
        PyRevertError { reason: err.output }
    }
}
