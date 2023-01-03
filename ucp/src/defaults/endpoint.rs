use std::convert::AsRef;
use std::marker::PhantomData;
use std::os::raw::{c_void, c_uint, c_char};
use nix::sys::socket::SockaddrLike;
use ucx2_sys::{
    ucp_ep_params_t,
    ucp_address_t,
    ucp_err_handling_mode_t,
    ucp_err_handler_t,
    ucp_err_handler_cb_t,
    ucs_sock_addr_t,
    ucp_conn_request_h,
    UCP_ERR_HANDLING_MODE_NONE,
};
use super::InternalDefault;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct EndpointParams<'a> {
    inner: ucp_ep_params_t,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> AsRef<ucp_ep_params_t> for EndpointParams<'a> {
    #[inline]
    fn as_ref(&self) -> &ucp_ep_params_t {
        &self.inner
    }
}

impl<'a> Default for EndpointParams<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: ucp_ep_params_t::default(),
            phantom_data: PhantomData,
        }
    }
}

impl<'a> EndpointParams<'a> {
    #[inline]
    pub fn field_mask(mut self, field_mask: u64) -> Self {
        self.inner.field_mask = field_mask;
        self
    }

    #[inline]
    pub fn address(mut self, address: *const ucp_address_t) -> Self {
        self.inner.address = address;
        self
    }

    #[inline]
    pub fn err_mode(mut self, err_mode: ucp_err_handling_mode_t) -> Self {
        self.inner.err_mode = err_mode;
        self
    }

    #[inline]
    pub fn err_handler(
        mut self,
        cb: ucp_err_handler_cb_t,
        arg: *mut c_void,
    ) -> Self {
        self.inner.err_handler.cb = cb;
        self.inner.err_handler.arg = arg;
        self
    }

    #[inline]
    pub fn user_data(mut self, user_data: *mut c_void) -> Self {
        self.inner.user_data = user_data;
        self
    }

    #[inline]
    pub fn flags(mut self, flags: c_uint) -> Self {
        self.inner.flags = flags;
        self
    }

    #[inline]
    pub fn sockaddr<S>(mut self, addr: &'a S) -> Self
    where
        S: SockaddrLike,
    {
        self.inner.sockaddr.addr = addr.as_ptr() as *const _;
        self.inner.sockaddr.addrlen = addr.len();
        self
    }

    #[inline]
    pub fn conn_request(mut self, conn_request: ucp_conn_request_h) -> Self {
        self.inner.conn_request = conn_request;
        self
    }

    #[inline]
    pub fn name(mut self, name: *const c_char) -> Self {
        self.inner.name = name;
        self
    }
}

impl InternalDefault for ucp_ep_params_t {
    #[inline]
    fn default() -> Self {
        Self {
            field_mask: 0,
            address: std::ptr::null(),
            err_mode: UCP_ERR_HANDLING_MODE_NONE,
            err_handler: ucp_err_handler_t::default(),
            user_data: std::ptr::null_mut(),
            flags: 0,
            sockaddr: ucs_sock_addr_t::default(),
            conn_request: std::ptr::null_mut(),
            name: std::ptr::null(),
        }
    }
}
