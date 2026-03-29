use crate::{
    error::ToPyError, event::Event, force_send::ForceSendExt, pusher::PyPusher, url_info::UrlInfo,
};
use fast_down_ffi::{BoxPusher, Error, Rx};
use parking_lot::Mutex;
use pyo3::prelude::*;
use std::{future::Future, sync::Arc};
use tokio_util::sync::CancellationToken;

#[pyclass]
pub struct DownloadTask {
    info: UrlInfo,
    task: Arc<fast_down_ffi::DownloadTask>,
    rx: Arc<Mutex<Option<Rx>>>,
    token: CancellationToken,
    child_token: Arc<Mutex<CancellationToken>>,
}

#[pymethods]
impl DownloadTask {
    #[getter]
    pub fn info(&self) -> UrlInfo {
        self.info.clone()
    }

    /// 彻底取消下载任务，不可恢复
    pub fn cancel(&self) {
        self.token.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// 暂停下载任务，可恢复
    pub fn pause(&self) {
        self.child_token.lock().cancel();
    }

    pub fn is_paused(&self) -> bool {
        self.child_token.lock().is_cancelled()
    }

    /// 开始下载任务写入到指定路径
    #[pyo3(signature = (save_path, callback=None))]
    pub fn start<'py>(
        &self,
        py: Python<'py>,
        save_path: String,
        callback: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rx = self.take_rx()?;
        let child_token = self.refresh_child_token();
        let task = self.task.clone();
        let rx_mutex = self.rx.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let download_fut = task.start(save_path.into(), child_token.clone());
            let (res, rx) = download_inner(download_fut, rx, callback)
                .force_send()
                .await;
            *rx_mutex.lock() = Some(rx);
            child_token.cancel();
            res
        })
    }

    /// 开始下载任务并返回内存中的数据
    #[pyo3(signature = (callback=None))]
    pub fn start_in_memory<'py>(
        &self,
        py: Python<'py>,
        callback: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rx = self.take_rx()?;
        let child_token = self.refresh_child_token();
        let task = self.task.clone();
        let rx_mutex = self.rx.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let download_fut = task.start_in_memory(child_token.clone());
            let (res, rx) = download_inner(download_fut, rx, callback)
                .force_send()
                .await;
            *rx_mutex.lock() = Some(rx);
            child_token.cancel();
            res
        })
    }

    /// 开始下载任务并使用自定义的 pusher
    #[pyo3(signature = (push_fn, flush_fn=None, callback=None))]
    pub fn start_with_pusher<'py>(
        &self,
        py: Python<'py>,
        push_fn: Py<PyAny>,
        flush_fn: Option<Py<PyAny>>,
        callback: Option<Py<PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let rx = self.take_rx()?;
        let child_token = self.refresh_child_token();
        let task = self.task.clone();
        let rx_mutex = self.rx.clone();
        let pusher = PyPusher::new(push_fn, flush_fn, task.config.write_buffer_size);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let download_fut = task.start_with_pusher(BoxPusher::new(pusher), child_token.clone());
            let (res, rx) = download_inner(download_fut, rx, callback)
                .force_send()
                .await;
            *rx_mutex.lock() = Some(rx);
            child_token.cancel();
            res
        })
    }
}

impl DownloadTask {
    pub fn new(task: fast_down_ffi::DownloadTask, rx: Rx, token: CancellationToken) -> Self {
        let child_token = token.child_token();
        child_token.cancel();
        Self {
            info: (&task.info).into(),
            task: Arc::new(task),
            rx: Arc::new(Mutex::new(Some(rx))),
            child_token: Arc::new(Mutex::new(child_token)),
            token,
        }
    }

    fn take_rx(&self) -> PyResult<Rx> {
        self.rx
            .lock()
            .take()
            .convert_err("Download task is running")
    }

    fn refresh_child_token(&self) -> CancellationToken {
        let child_token = self.token.child_token();
        *self.child_token.lock() = child_token.clone();
        child_token
    }
}

async fn download_inner<R>(
    download_fut: impl Future<Output = Result<R, Error>>,
    rx: Rx,
    callback: Option<Py<PyAny>>,
) -> (PyResult<R>, Rx) {
    tokio::pin!(download_fut);
    let res = loop {
        tokio::select! {
            res = &mut download_fut => break res,
            event = rx.recv() => {
                match event {
                    Ok(e) => {
                        if let Some(ref cb) = callback {
                            Python::attach(|py| {
                                let _ = cb.bind(py).call1((Event::from(e),));
                            });
                        }
                    }
                    Err(_) => break download_fut.await,
                }
            }
        }
    };
    if let Some(ref cb) = callback {
        Python::attach(|py| {
            while let Ok(e) = rx.try_recv() {
                let _ = cb.bind(py).call1((Event::from(e),));
            }
        });
    } else {
        while rx.try_recv().is_ok() {}
    }
    let res = res.convert_err("Download task failed");
    (res, rx)
}
