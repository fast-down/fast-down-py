use crate::download::DownloadTask;
use crate::{cancel::CancellationToken, config::Config, error::ToPyError};
use fast_down_ffi::create_channel;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use url::Url;

#[pyfunction]
#[pyo3(signature = (url, config=None, token=None))]
#[allow(clippy::needless_pass_by_value)]
pub fn prefetch<'py>(
    py: Python<'py>,
    url: String,
    config: Option<Config>,
    token: Option<PyRef<CancellationToken>>,
) -> PyResult<Bound<'py, PyAny>> {
    let url: Url = url.parse().convert_err("Invalid URL")?;
    let config = config.map(|c| c.to_ffi_config()).unwrap_or_default();
    let token = token.map(|t| t.token.clone()).unwrap_or_default();
    let (tx, rx) = create_channel();
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        let task = tokio::select! {
            () = token.cancelled() => {
                return Err(PyRuntimeError::new_err("Prefetch Cancelled"));
            }
            t = fast_down_ffi::prefetch(url, config, tx) => {
                t.convert_err("Prefetch Failed")?
            }
        };
        Ok(DownloadTask::new(task, rx, token))
    })
}
