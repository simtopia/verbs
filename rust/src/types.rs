use pyo3::exceptions::PyException;
use pyo3::PyErr;
use revm::primitives::ExecutionResult;
use rust_sim::{contract::Event, network::RevertError};
use std::fmt;

pub type PyAddress = [u8; 20];

pub type PyLog = (PyAddress, Vec<u8>);

pub type PyEvent = (String, Vec<PyLog>, usize, usize);

pub type PyExecutionResult = (Option<Vec<u8>>, Vec<PyLog>, u64);

pub fn convert_event(event: &Event) -> PyEvent {
    (
        event.function_name.to_string(),
        event
            .logs
            .iter()
            .map(|x| (x.address.0 .0, x.data.to_vec()))
            .collect(),
        event.step,
        event.sequence,
    )
}

pub fn result_to_py(
    result: Result<ExecutionResult, RevertError>,
) -> Result<PyExecutionResult, RevertError> {
    match result {
        Ok(x) => Ok((
            x.output().map(|b| b.to_vec()),
            x.logs()
                .into_iter()
                .map(|a| (a.address.0 .0, a.data.to_vec()))
                .collect(),
            x.gas_used(),
        )),
        Err(x) => Err(x),
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
