use pyo3::prelude::*;

#[pyclass(skip_from_py_object, get_all, set_all)]
#[derive(Debug, Clone)]
pub struct UrlInfo {
    pub size: u64,
    pub raw_name: String,
    pub supports_range: bool,
    pub fast_download: bool,
    pub final_url: String,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub content_type: Option<String>,
}

#[pymethods]
impl UrlInfo {
    pub fn filename(&self) -> String {
        sanitize_filename::sanitize_with_options(
            &self.raw_name,
            sanitize_filename::Options {
                windows: cfg!(windows),
                truncate: true,
                replacement: "_",
            },
        )
    }

    fn __repr__(&self) -> String {
        format!("<UrlInfo name='{}' size={}>", self.raw_name, self.size)
    }
}

impl From<&fast_down_ffi::UrlInfo> for UrlInfo {
    fn from(v: &fast_down_ffi::UrlInfo) -> Self {
        Self {
            size: v.size,
            raw_name: v.raw_name.clone(),
            supports_range: v.supports_range,
            fast_download: v.fast_download,
            final_url: v.final_url.to_string(),
            etag: v.file_id.etag.as_ref().map(ToString::to_string),
            last_modified: v.file_id.last_modified.as_ref().map(ToString::to_string),
            content_type: v.content_type.clone(),
        }
    }
}
