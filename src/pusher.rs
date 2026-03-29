use bytes::{Bytes, BytesMut};
use fast_down_ffi::ProgressEntry;
use pyo3::{prelude::*, types::PyBytes};
use std::collections::BTreeMap;

pub struct PyPusher {
    pub push_fn: Py<PyAny>,
    pub flush_fn: Option<Py<PyAny>>,
    pub cache: BTreeMap<u64, Bytes>,
    pub cache_size: usize,
    pub buffer_size: usize,
}

impl PyPusher {
    pub const fn new(push_fn: Py<PyAny>, flush_fn: Option<Py<PyAny>>, buffer_size: usize) -> Self {
        Self {
            push_fn,
            flush_fn,
            cache: BTreeMap::new(),
            cache_size: 0,
            buffer_size,
        }
    }

    fn send_to_py(push_fn: &Py<PyAny>, start: u64, content: &[u8]) -> Result<(), String> {
        Python::attach(|py| -> Result<(), String> {
            let py_data = PyBytes::new(py, content);
            push_fn
                .bind(py)
                .call1((start, py_data))
                .map_err(|e| format!("Python push error: {e}"))
                .map(|_| ())
        })?;
        Ok(())
    }

    fn flush_buffer(&mut self) -> Result<(), String> {
        let mut merged_start: Option<u64> = None;
        let mut merged_bytes = BytesMut::new();
        while let Some((start, chunk)) = self.cache.pop_first() {
            let len = chunk.len();
            self.cache_size -= len;
            if let Some(m_start) = merged_start {
                if m_start + (merged_bytes.len() as u64) == start {
                    merged_bytes.extend_from_slice(&chunk);
                    continue;
                }
                let data_to_send = merged_bytes.split().freeze();
                if let Err(e) = Self::send_to_py(&self.push_fn, m_start, &data_to_send) {
                    let len_to_send = data_to_send.len();
                    self.cache.insert(m_start, data_to_send);
                    self.cache.insert(start, chunk);
                    self.cache_size += len_to_send + len;
                    return Err(e);
                }
            }
            merged_start = Some(start);
            merged_bytes.extend_from_slice(&chunk);
        }
        if let Some(m_start) = merged_start
            && !merged_bytes.is_empty()
        {
            let data_to_send = merged_bytes.freeze();
            if let Err(e) = Self::send_to_py(&self.push_fn, m_start, &data_to_send) {
                let len_to_send = data_to_send.len();
                self.cache.insert(m_start, data_to_send);
                self.cache_size += len_to_send;
                return Err(e);
            }
        }
        Ok(())
    }
}

impl fast_down_ffi::Pusher for PyPusher {
    type Error = String;

    fn push(&mut self, range: &ProgressEntry, content: Bytes) -> Result<(), (Self::Error, Bytes)> {
        let start = range.start;
        let new_len = content.len();
        match self.cache.get(&start) {
            Some(old) if new_len <= old.len() => return Ok(()),
            Some(old) => self.cache_size -= old.len(),
            None => {}
        }
        self.cache.insert(start, content);
        self.cache_size += new_len;
        if self.cache_size >= self.buffer_size {
            self.flush_buffer().map_err(|e| {
                let failed_bytes = self.cache.remove(&range.start).unwrap_or_default();
                self.cache_size -= failed_bytes.len();
                (e, failed_bytes)
            })?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_buffer()?;
        if let Some(flush_fn) = &self.flush_fn {
            Python::attach(|py| -> Result<(), String> {
                flush_fn
                    .call0(py)
                    .map_err(|e| format!("Python flush error: {e}"))
                    .map(|_| ())
            })?;
        }
        Ok(())
    }
}
