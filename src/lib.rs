mod cancel;
mod config;
mod download;
mod error;
mod event;
mod force_send;
mod prefetch;
mod pusher;
mod url_info;

#[pyo3::pymodule]
mod fastdown {
    #[pymodule_export]
    use super::cancel::CancellationToken;
    #[pymodule_export]
    use super::config::Config;
    #[pymodule_export]
    use super::download::DownloadTask;
    #[pymodule_export]
    use super::event::Event;
    #[pymodule_export]
    use super::prefetch::prefetch;
    #[pymodule_export]
    use super::url_info::UrlInfo;
}
