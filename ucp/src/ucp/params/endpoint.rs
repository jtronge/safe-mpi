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
    ucs_status_t,
    UCP_ERR_HANDLING_MODE_NONE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE,
    UCP_EP_PARAM_FIELD_ERR_HANDLER,
    UCP_EP_PARAM_FIELD_USER_DATA,
    UCP_EP_PARAM_FIELD_FLAGS,
    UCP_EP_PARAM_FIELD_SOCK_ADDR,
    UCP_EP_PARAM_FIELD_CONN_REQUEST,
    UCP_EP_PARAM_FIELD_NAME,
};
use crate::Status;
use crate::ucp::{
    Endpoint,
    ConnRequest,
};
use crate::ucp::callbacks::err_handler_cb;
use super::InternalDefault;

#[repr(transparent)]
#[derive(Debug)]
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
    pub fn address(mut self, address: *const ucp_address_t) -> Self {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_REMOTE_ADDRESS as u64;
        self.inner.address = address;
        self
    }

    #[inline]
    pub fn err_mode(mut self, err_mode: ucp_err_handling_mode_t) -> Self {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE as u64;
        self.inner.err_mode = err_mode;
        self
    }

    #[inline]
    pub fn err_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(Endpoint, Status),
    {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_ERR_HANDLER as u64;
        let f: Box<dyn Fn(Endpoint, Status)> = Box::new(f);
        let arg = Box::into_raw(Box::new(f));
        self.inner.err_handler.cb = Some(err_handler_cb);
        self.inner.err_handler.arg = arg as *mut _;
        self
    }

    #[inline]
    pub fn user_data(mut self, user_data: *mut c_void) -> Self {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_USER_DATA as u64;
        self.inner.user_data = user_data;
        self
    }

    #[inline]
    pub fn flags(mut self, flags: c_uint) -> Self {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_FLAGS as u64;
        self.inner.flags = flags;
        self
    }

    #[inline]
    pub fn sockaddr<S>(mut self, addr: &'a S) -> Self
    where
        S: SockaddrLike,
    {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_SOCK_ADDR as u64;
        self.inner.sockaddr.addr = addr.as_ptr() as *const _;
        self.inner.sockaddr.addrlen = addr.len();
        self
    }

    #[inline]
    pub fn conn_request(mut self, conn_request: ConnRequest) -> Self {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_CONN_REQUEST as u64;
        self.inner.conn_request = conn_request.into_raw();
        self
    }

    #[inline]
    pub fn name(mut self, name: *const c_char) -> Self {
        self.inner.field_mask |= UCP_EP_PARAM_FIELD_NAME as u64;
        self.inner.name = name;
        self
    }
}

impl<'a> Drop for EndpointParams<'a> {
    fn drop(&mut self) {
        unsafe {
            if let Some(_) = self.inner.err_handler.cb {
                let _ = Box::from_raw(
                    self.inner.err_handler.arg as *mut Box<
                        dyn Fn(Endpoint, Status)
                    >
                );
            }
        }
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
