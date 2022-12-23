use std::default::Default;
use std::os::raw::c_void;
use std::marker::PhantomData;
use std::convert::AsRef;
use nix::sys::socket::SockaddrLike;
use ucx2_sys::{
    ucp_listener_params_t,
    ucs_sock_addr_t,
    ucp_listener_accept_handler_t,
    ucp_listener_accept_callback_t,
    ucp_listener_conn_handler_t,
    ucp_listener_conn_callback_t,
};
use super::InternalDefault;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct ListenerParams<'a> {
    inner: ucp_listener_params_t,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> AsRef<ucp_listener_params_t> for ListenerParams<'a> {
    fn as_ref(&self) -> &ucp_listener_params_t {
        &self.inner
    }
}

impl<'a> Default for ListenerParams<'a> {
    #[inline]
    fn default() -> Self {
        Self {
            inner: ucp_listener_params_t::default(),
            phantom_data: PhantomData,
        }
    }
}

impl<'a> ListenerParams<'a> {
    #[inline]
    pub fn field_mask(mut self, field_mask: u64) -> Self {
        self.inner.field_mask = field_mask;
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
    pub fn accept_handler(
        mut self,
        cb: ucp_listener_accept_callback_t,
        arg: *mut c_void,
    ) -> Self {
        self.inner.accept_handler.cb = cb;
        self.inner.accept_handler.arg = arg;
        self
    }

    #[inline]
    pub fn conn_handler(
        mut self,
        cb: ucp_listener_conn_callback_t,
        arg: *mut c_void,
    ) -> Self {
        self.inner.conn_handler.cb = cb;
        self.inner.conn_handler.arg = arg;
        self
    }
}

impl InternalDefault for ucp_listener_accept_handler_t {
    fn default() -> Self {
        Self {
            cb: None,
            arg: std::ptr::null_mut(),
        }
    }
}

impl InternalDefault for ucp_listener_conn_handler_t {
    fn default() -> Self {
        Self {
            cb: None,
            arg: std::ptr::null_mut(),
        }
    }
}

impl InternalDefault for ucp_listener_params_t {
    fn default() -> Self {
        Self {
            field_mask: 0,
            sockaddr: ucs_sock_addr_t::default(),
            accept_handler: ucp_listener_accept_handler_t::default(),
            conn_handler: ucp_listener_conn_handler_t::default(),
        }
    }
}
