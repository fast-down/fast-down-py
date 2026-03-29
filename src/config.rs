use fast_down_ffi::Proxy;
use parking_lot::lock_api::Mutex;
use pyo3::prelude::*;
use std::{collections::HashMap, sync::Arc, time::Duration};

#[pyclass(from_py_object, get_all, set_all)]
#[derive(Clone, Default)]
pub struct Config {
    pub threads: Option<usize>,
    pub proxy: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub min_chunk_size: Option<u64>,
    pub write_buffer_size: Option<usize>,
    pub write_queue_cap: Option<usize>,
    pub retry_gap_ms: Option<u64>,
    pub pull_timeout_ms: Option<u64>,
    pub accept_invalid_certs: Option<bool>,
    pub accept_invalid_hostnames: Option<bool>,
    pub write_method: Option<String>,
    pub retry_times: Option<usize>,
    pub local_address: Option<Vec<String>>,
    pub max_speculative: Option<usize>,
    pub downloaded_chunk: Option<Vec<(u64, u64)>>,
    pub chunk_window: Option<u64>,
}

#[pymethods]
impl Config {
    #[new]
    #[pyo3(signature = (
        threads=None, proxy=None, headers=None, min_chunk_size=None,
        write_buffer_size=None, write_queue_cap=None, retry_gap_ms=None,
        pull_timeout_ms=None, accept_invalid_certs=None, accept_invalid_hostnames=None,
        write_method=None, retry_times=None, local_address=None,
        max_speculative=None, downloaded_chunk=None, chunk_window=None
    ))]
    #[allow(clippy::too_many_arguments)]
    const fn new(
        threads: Option<usize>,
        proxy: Option<String>,
        headers: Option<HashMap<String, String>>,
        min_chunk_size: Option<u64>,
        write_buffer_size: Option<usize>,
        write_queue_cap: Option<usize>,
        retry_gap_ms: Option<u64>,
        pull_timeout_ms: Option<u64>,
        accept_invalid_certs: Option<bool>,
        accept_invalid_hostnames: Option<bool>,
        write_method: Option<String>,
        retry_times: Option<usize>,
        local_address: Option<Vec<String>>,
        max_speculative: Option<usize>,
        downloaded_chunk: Option<Vec<(u64, u64)>>,
        chunk_window: Option<u64>,
    ) -> Self {
        Self {
            threads,
            proxy,
            headers,
            min_chunk_size,
            write_buffer_size,
            write_queue_cap,
            retry_gap_ms,
            pull_timeout_ms,
            accept_invalid_certs,
            accept_invalid_hostnames,
            write_method,
            retry_times,
            local_address,
            max_speculative,
            downloaded_chunk,
            chunk_window,
        }
    }
}

impl Config {
    pub fn to_ffi_config(&self) -> fast_down_ffi::Config {
        fast_down_ffi::Config {
            threads: self.threads.unwrap_or(32),
            proxy: match self.proxy.as_deref() {
                Some("no") => Proxy::No,
                Some("system") | None => Proxy::System,
                Some(p) => Proxy::Custom(p.to_string()),
            },
            headers: self.headers.clone().unwrap_or_default(),
            min_chunk_size: self.min_chunk_size.unwrap_or(500 * 1024),
            write_buffer_size: self.write_buffer_size.unwrap_or(16 * 1024 * 1024),
            write_queue_cap: self.write_queue_cap.unwrap_or(10240),
            retry_gap: Duration::from_millis(self.retry_gap_ms.unwrap_or(500)),
            pull_timeout: Duration::from_millis(self.pull_timeout_ms.unwrap_or(5000)),
            accept_invalid_certs: self.accept_invalid_certs.unwrap_or(false),
            accept_invalid_hostnames: self.accept_invalid_hostnames.unwrap_or(false),
            write_method: match self.write_method.as_deref().unwrap_or_default() {
                "std" => fast_down_ffi::WriteMethod::Std,
                _ => fast_down_ffi::WriteMethod::Mmap,
            },
            retry_times: self.retry_times.unwrap_or(3),
            local_address: self
                .local_address
                .as_ref()
                .map(|e| e.iter().filter_map(|p| p.parse().ok()).collect())
                .unwrap_or_default(),
            max_speculative: self.max_speculative.unwrap_or(3),
            #[allow(clippy::cast_sign_loss)]
            downloaded_chunk: Arc::new(Mutex::new(
                self.downloaded_chunk
                    .as_ref()
                    .map(|e| e.iter().map(|p| p.0..p.1).collect())
                    .unwrap_or_default(),
            )),
            chunk_window: self.chunk_window.unwrap_or(8 * 1024),
        }
    }
}
