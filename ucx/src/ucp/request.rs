use ucx2_sys::{
    ucp_request_check_status,
    ucp_request_free,
};
use crate::Status;
use std::marker::PhantomData;
use std::os::raw::c_void;

pub struct Request<'a> {
    req_ptr: Option<*mut c_void>,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> Request<'a> {
    pub fn from_raw(req_ptr: *mut c_void) -> Request<'a> {
        Request {
            req_ptr: Some(req_ptr),
            _phantom_data: PhantomData,
        }
    }

    pub unsafe fn status(&self) -> Status {
        Status::from_raw(ucp_request_check_status(self.req_ptr.unwrap()))
    }

    /// Take ownership of the request and free it.
    pub unsafe fn free(self) {
        ucp_request_free(self.req_ptr.unwrap());
    }
}