use ucx2_sys::{
    ucs_sock_addr_t,
    ucs_cpu_set_t,
    ucp_err_handler_t,
};
mod listener;
pub use listener::ListenerParams;
mod worker;
pub use worker::WorkerParams;
mod endpoint;
pub use endpoint::EndpointParams;
mod request;
pub use request::RequestParam;

/// Internal default trait to be applied to external ucx2_sys code.
pub(crate) trait InternalDefault {
    fn default() -> Self;
}

impl InternalDefault for ucs_sock_addr_t {
    #[inline]
    fn default() -> Self {
        Self {
            addr: std::ptr::null(),
            addrlen: 0,
        }
    }
}

impl InternalDefault for ucs_cpu_set_t {
    #[inline]
    fn default() -> Self {
        Self {
            ucs_bits: [0; 16],
        }
    }
}

impl InternalDefault for ucp_err_handler_t {
    #[inline]
    fn default() -> Self {
        Self {
            cb: None,
            arg: std::ptr::null_mut(),
        }
    }
}
