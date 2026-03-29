use pyo3::prelude::*;

#[derive(Debug, Default, Clone)]
#[pyclass(skip_from_py_object)]
pub struct CancellationToken {
    pub(crate) token: tokio_util::sync::CancellationToken,
}

#[pymethods]
impl CancellationToken {
    #[new]
    pub fn new() -> Self {
        let token = tokio_util::sync::CancellationToken::new();
        Self { token }
    }

    pub fn cancel(&self) {
        self.token.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }
}
