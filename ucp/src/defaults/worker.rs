use std::convert::AsRef;
use std::default::Default;
use ucx2_sys::{
    ucp_worker_params_t,
    ucs_thread_mode_t,
    UCS_THREAD_MODE_SINGLE,
    ucs_cpu_set_t,
};
use super::InternalDefault;
use std::os::raw::{
    c_uint,
    c_void,
    c_int,
    c_char,
};

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct WorkerParams {
    inner: ucp_worker_params_t,
}

impl AsRef<ucp_worker_params_t> for WorkerParams {
    #[inline]
    fn as_ref(&self) -> &ucp_worker_params_t {
        &self.inner
    }
}

impl Default for WorkerParams {
    #[inline]
    fn default() -> Self {
        Self {
            inner: ucp_worker_params_t::default(),
        }
    }
}

impl WorkerParams {
    #[inline]
    pub fn field_mask(mut self, field_mask: u64) -> Self {
        self.inner.field_mask = field_mask;
        self
    }

    #[inline]
    pub fn thread_mode(mut self, thread_mode: ucs_thread_mode_t) -> Self {
        self.inner.thread_mode = thread_mode;
        self
    }

    #[inline]
    pub fn cpu_mask(mut self, cpu_mask: ucs_cpu_set_t) -> Self {
        self.inner.cpu_mask = cpu_mask;
        self
    }

    #[inline]
    pub fn events(mut self, events: c_uint) -> Self {
        self.inner.events = events;
        self
    }

    #[inline]
    pub fn user_data(mut self, user_data: *mut c_void) -> Self {
        self.inner.user_data = user_data;
        self
    }

    #[inline]
    pub fn event_fd(mut self, event_fd: c_int) -> Self {
        self.inner.event_fd = event_fd;
        self
    }

    #[inline]
    pub fn flags(mut self, flags: u64) -> Self {
        self.inner.flags = flags;
        self
    }

    #[inline]
    pub fn name(mut self, name: *const c_char) -> Self {
        self.inner.name = name;
        self
    }

    #[inline]
    pub fn am_alignment(mut self, am_alignment: usize) -> Self {
        self.inner.am_alignment = am_alignment;
        self
    }

    #[inline]
    pub fn client_id(mut self, client_id: u64) -> Self {
        self.inner.client_id = client_id;
        self
    }
}

impl InternalDefault for ucp_worker_params_t {
    #[inline]
    fn default() -> Self {
        Self {
            field_mask: 0,
            thread_mode: UCS_THREAD_MODE_SINGLE,
            cpu_mask: ucs_cpu_set_t::default(),
            events: 0,
            user_data: std::ptr::null_mut(),
            event_fd: 0,
            flags: 0,
            name: std::ptr::null(),
            am_alignment: 0,
            client_id: 0,
        }
    }
}
