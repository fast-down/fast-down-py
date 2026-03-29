use pyo3::prelude::*;

#[pyclass(skip_from_py_object, get_all, set_all)]
#[derive(Debug, Clone)]
pub struct Event {
    #[pyo3(name = "type")]
    #[allow(clippy::struct_field_names)]
    pub event_type: String,
    pub id: Option<usize>,
    pub message: Option<String>,
    pub range: Option<(u64, u64)>,
}

#[pymethods]
impl Event {
    fn __repr__(&self) -> String {
        format!(
            "Event(type='{}', id={:?}, message='{:?}', range={:?})",
            self.event_type, self.id, self.message, self.range
        )
    }
}

impl From<fast_down_ffi::Event> for Event {
    fn from(value: fast_down_ffi::Event) -> Self {
        let mut event = Self {
            event_type: String::with_capacity(20),
            id: None,
            message: None,
            range: None,
        };
        match value {
            fast_down_ffi::Event::PrefetchError(e) => {
                event.event_type.push_str("PrefetchError");
                event.message = Some(e);
            }
            fast_down_ffi::Event::Pulling(id) => {
                event.event_type.push_str("Pulling");
                event.id = Some(id);
            }
            fast_down_ffi::Event::PullError(id, e) => {
                event.event_type.push_str("PullError");
                event.id = Some(id);
                event.message = Some(e);
            }
            fast_down_ffi::Event::PullTimeout(id) => {
                event.event_type.push_str("PullTimeout");
                event.id = Some(id);
            }
            fast_down_ffi::Event::PullProgress(id, range) => {
                event.event_type.push_str("PullProgress");
                event.id = Some(id);
                event.range = Some((range.start, range.end));
            }
            fast_down_ffi::Event::Pushing(id, range) => {
                event.event_type.push_str("Pushing");
                event.id = Some(id);
                event.range = Some((range.start, range.end));
            }
            fast_down_ffi::Event::PushError(id, range, e) => {
                event.event_type.push_str("PushError");
                event.id = Some(id);
                event.message = Some(e);
                event.range = Some((range.start, range.end));
            }
            fast_down_ffi::Event::PushProgress(id, range) => {
                event.event_type.push_str("PushProgress");
                event.id = Some(id);
                event.range = Some((range.start, range.end));
            }
            fast_down_ffi::Event::Flushing => {
                event.event_type.push_str("Flushing");
            }
            fast_down_ffi::Event::FlushError(e) => {
                event.event_type.push_str("FlushError");
                event.message = Some(e);
            }
            fast_down_ffi::Event::Finished(id) => {
                event.event_type.push_str("Finished");
                event.id = Some(id);
            }
        }
        event
    }
}
