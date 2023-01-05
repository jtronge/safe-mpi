use ucx2_sys::{
    ucs_status_ptr_t,
    ucp_request_check_status,
    ucp_request_free,
    ucs_status_t,
};
use crate::RequestParam;
use std::marker::PhantomData;
use std::os::raw::c_void;

pub struct Request<'a> {
    req_ptr: Option<*mut c_void>,
    param: Option<RequestParam>,
    phantom_data: PhantomData<&'a ()>,
}

impl<'a> Request<'a> {
    /// TODO: Remove this
    pub fn new() -> Request<'a> {
        Request {
            req_ptr: None,
            param: None,
            phantom_data: PhantomData,
        }
    }

    pub fn from_raw(req_ptr: *mut c_void, param: RequestParam) -> Request<'a> {
        Request {
            req_ptr: Some(req_ptr),
            param: Some(param),
            phantom_data: PhantomData,
        }
    }

    pub unsafe fn status(&self) -> ucs_status_t {
        ucp_request_check_status(self.req_ptr.unwrap())
    }

    /// Take ownership of the request and free it.
    pub unsafe fn free(self) {
        ucp_request_free(self.req_ptr.unwrap());
    }
}
