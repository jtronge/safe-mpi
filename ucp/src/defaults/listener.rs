use std::convert::AsRef;
use std::default::Default;
use std::marker::PhantomData;
use std::os::raw::c_void;
use nix::sys::socket::SockaddrLike;
use ucx2_sys::{
    ucp_listener_params_t,
    ucs_sock_addr_t,
    ucp_listener_accept_handler_t,
    ucp_listener_accept_callback_t,
    ucp_listener_conn_handler_t,
    ucp_listener_conn_callback_t,
};
use crate::{
    Endpoint,
    ConnRequest,
};
use crate::callbacks::{
    listener_accept_callback,
    listener_conn_callback,
};
use super::InternalDefault;

#[repr(transparent)]
#[derive(Debug)]
pub struct ListenerParams<'a> {
    inner: ucp_listener_params_t,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> AsRef<ucp_listener_params_t> for ListenerParams<'a> {
    #[inline]
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

    // TODO: Modify these to use closures here
    #[inline]
    pub fn accept_handler<F>(
        mut self,
        f: F,
    ) -> Self
    where
        F: Fn(Endpoint),
    {
        let f: Box<dyn Fn(Endpoint)> = Box::new(f);
        let arg = Box::into_raw(Box::new(f));
        self.inner.accept_handler.cb = Some(listener_accept_callback);
        self.inner.accept_handler.arg = arg as *mut _;
        self
    }

    #[inline]
    pub fn conn_handler<F>(
        mut self,
        f: F,
    ) -> Self
    where
        F: Fn(ConnRequest),
    {
        let f: Box<dyn Fn(ConnRequest)> = Box::new(f);
        let arg = Box::into_raw(Box::new(f));
        self.inner.conn_handler.cb = Some(listener_conn_callback);
        self.inner.conn_handler.arg = arg as *mut _;
        println!("arg ptr: {}", arg as usize);
        self
    }
}

impl<'a> Drop for ListenerParams<'a> {
    fn drop(&mut self) {
        // TODO
        unsafe {
            if let Some(_) = self.inner.accept_handler.cb {
                let _ = Box::from_raw(
                    self.inner.accept_handler.arg as *mut Box<dyn Fn(Endpoint)>
                );
            }
            if let Some(_) = self.inner.conn_handler.cb {
                println!("arg ptr: {}", self.inner.conn_handler.arg as usize);
                let _ = Box::from_raw(
                    self.inner.conn_handler.arg as *mut Box<dyn Fn(ConnRequest)>
                );
            }
        }
    }
}

impl InternalDefault for ucp_listener_accept_handler_t {
    #[inline]
    fn default() -> Self {
        Self {
            cb: None,
            arg: std::ptr::null_mut(),
        }
    }
}

impl InternalDefault for ucp_listener_conn_handler_t {
    #[inline]
    fn default() -> Self {
        Self {
            cb: None,
            arg: std::ptr::null_mut(),
        }
    }
}

impl InternalDefault for ucp_listener_params_t {
    #[inline]
    fn default() -> Self {
        Self {
            field_mask: 0,
            sockaddr: ucs_sock_addr_t::default(),
            accept_handler: ucp_listener_accept_handler_t::default(),
            conn_handler: ucp_listener_conn_handler_t::default(),
        }
    }
}
