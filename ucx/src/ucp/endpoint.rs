use ucx2_sys::{
    ucp_ep_h,
    ucp_ep_create,
    ucp_ep_close_nbx,
    ucp_request_check_status,
    ucp_request_free,
    ucp_tag_t,
    UCS_OK,
    ucp_tag_send_nbx,
    ucp_tag_recv_nbx,
    ucp_stream_send_nbx,
    ucp_stream_recv_nbx,
    rust_ucs_ptr_is_err,
    rust_ucs_ptr_is_ptr,
    rust_ucs_ptr_status,
    UCS_INPROGRESS,
};
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::marker::PhantomData;
use super::{Worker, Request, RequestParam, EndpointParams};
use crate::{Status, Result};

pub enum StreamRecvResult<'a> {
    Running(Request<'a>),
    Complete(usize),
}

#[repr(transparent)]
pub struct Endpoint<'a> {
    ep: ucp_ep_h,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> Endpoint<'a> {
    /// Create the endpoint for the worker and params.
    pub fn new(worker: Worker, params: &EndpointParams<'a>) -> Endpoint<'a> {
        let mut ep = MaybeUninit::<ucp_ep_h>::uninit();
        let status = unsafe {
            ucp_ep_create(worker.into_raw(), params.as_ref(), ep.as_mut_ptr())
        };
        if status != UCS_OK {
            panic!("ucp_ep_create() failed");
        }
        let ep = unsafe { ep.assume_init() };
        Endpoint {
            ep,
            _phantom_data: PhantomData,
        }
    }

    /// Create a new endpoint from a raw ucp endpoint.
    pub fn from_raw(ep: ucp_ep_h) -> Endpoint<'a> {
        Endpoint {
            ep,
            _phantom_data: PhantomData,
        }
    }

    unsafe fn check_req_err<'b>(
        req_ptr: *mut c_void,
    ) -> Result<Option<Request<'b>>> {
        if req_ptr == std::ptr::null_mut() {
            // Request already completed
            Ok(None)
        } else if rust_ucs_ptr_is_err(req_ptr) == 0 {
            // Request still running
            Ok(Some(Request::from_raw(req_ptr)))
        } else {
            // Error occurred
            Err(Status::from_raw(rust_ucs_ptr_status(req_ptr)))
        }
    }

    /// Do a non-blocking tagged send.
    pub unsafe fn tag_send_nbx<'b>(
        &self,
        buf: &'b [u8],
        tag: ucp_tag_t,
        param: &RequestParam,
    ) -> Result<Option<Request<'b>>> {
        // TODO: Is there a way to check for a CONTIGUOUS-like datatype?
        // assert_eq!(param.as_ref().datatype, UCP_DATATYPE_CONTIG.into());
        let req_ptr = ucp_tag_send_nbx(self.ep, buf.as_ptr() as *const _,
                                       buf.len() * std::mem::size_of::<u8>(),
                                       tag, param.as_ref());
        Self::check_req_err(req_ptr)
    }

    /// Do a non-blocking tagged receive.
    pub unsafe fn tag_recv_nbx<'b>(
        &self,
        worker: Worker,
        buf: &'b mut [u8],
        tag: ucp_tag_t,
        param: &RequestParam,
    ) -> Result<Option<Request<'b>>> {
        let req_ptr = ucp_tag_recv_nbx(worker.into_raw(),
                                       buf.as_mut_ptr() as *mut _,
                                       buf.len() * std::mem::size_of::<u8>(),
                                       tag, 0, param.as_ref());
        Self::check_req_err(req_ptr)
    }

    /// Do a non-blocking streamed send.
    pub unsafe fn stream_send_nbx<'b>(
        &self,
        buf: &'b [u8],
        param: &RequestParam,
    ) -> Result<Option<Request<'b>>> {
        let req_ptr = ucp_stream_send_nbx(self.ep, buf.as_ptr() as *const _,
                                          buf.len() * std::mem::size_of::<u8>(),
                                          param.as_ref());
        Self::check_req_err(req_ptr)
    }

    /// Do a non-blocking streamed recv.
    pub unsafe fn stream_recv_nbx<'b>(
        &self,
        buf: &'b mut [u8],
        param: &RequestParam,
    ) -> Result<StreamRecvResult<'b>> {
        // Note that the length pointer is only valid if the receive operation
        // completes immediately (so the return value is NULL -- represented
        // here as None).
        let mut length = 0;
        let req_ptr = ucp_stream_recv_nbx(self.ep, buf.as_mut_ptr() as *mut _,
                                          buf.len() * std::mem::size_of::<u8>(),
                                          &mut length, param.as_ref());
        Self::check_req_err(req_ptr)
            .map(|req| {
                if let Some(req) = req {
                    StreamRecvResult::Running(req)
                } else {
                    StreamRecvResult::Complete(length)
                }
            })
    }

    /// Close the endpoint.
    pub unsafe fn close(self, worker: Worker, flags: u32) -> Status {
        let param = RequestParam::default()
            .flags(flags);
        let close_req = ucp_ep_close_nbx(self.ep, param.as_ref());
        let status = if rust_ucs_ptr_is_ptr(close_req) != 0 {
            let mut status = UCS_OK;
            loop {
                worker.progress();
                status = ucp_request_check_status(close_req);
                if status != UCS_INPROGRESS {
                    break;
                }
            }
            ucp_request_free(close_req);
            status
        } else {
            rust_ucs_ptr_status(close_req)
        };
        Status::from_raw(status)
    }
}