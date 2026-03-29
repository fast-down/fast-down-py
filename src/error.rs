use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use std::fmt::Display;

pub trait ToPyError<T> {
    fn convert_err(self, topic: &str) -> PyResult<T>;
}

impl<T, E: Display> ToPyError<T> for Result<T, E> {
    fn convert_err(self, topic: &str) -> PyResult<T> {
        self.map_err(|err| PyRuntimeError::new_err(format!("{topic}: {err}")))
    }
}

impl<T> ToPyError<T> for Option<T> {
    fn convert_err(self, topic: &str) -> PyResult<T> {
        self.ok_or_else(|| PyRuntimeError::new_err(topic.to_string()))
    }
}
