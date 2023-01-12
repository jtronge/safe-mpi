use ucx2_sys::{
    ucp_ep_h,
    ucp_ep_create,
    ucp_ep_close_nbx,
    ucp_request_check_status,
    ucp_request_free,
    ucp_tag_t,
    ucs_status_t,
    UCS_OK,
    ucp_tag_send_nbx,
    ucp_tag_recv_nbx,
    rust_ucs_ptr_is_err,
    rust_ucs_ptr_is_ptr,
    rust_ucs_ptr_status,
    ucs_status_string,
    UCP_DATATYPE_CONTIG,
    UCP_OP_ATTR_FIELD_FLAGS,
    UCP_EP_CLOSE_MODE_FLUSH,
    UCS_INPROGRESS,
};
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use super::{Worker, Request, RequestParam, EndpointParams};
use crate::status_to_string;

pub struct Endpoint<'a> {
    ep: ucp_ep_h,
    params: Option<EndpointParams<'a>>,
}

impl<'a> Endpoint<'a> {
    /// Create the endpoint for the worker and params.
    pub fn new(worker: Worker, params: EndpointParams<'a>) -> Endpoint<'a> {
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
            params: Some(params),
        }
    }

    /// Create a new endpoint from a raw ucp endpoint.
    pub fn from_raw<'b>(ep: ucp_ep_h) -> Endpoint<'b> {
        Endpoint {
            ep,
            params: None,
        }
    }

    unsafe fn check_req_err(req_ptr: *mut c_void) -> Result<(), ()> {
        if req_ptr == std::ptr::null_mut()
           || rust_ucs_ptr_is_err(req_ptr) != 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Do a non-blocking tagged send.
    pub unsafe fn tag_send_nbx<'b>(
        &self,
        buf: &'b [u8],
        tag: ucp_tag_t,
        param: RequestParam,
    ) -> Result<Request<'b>, ()> {
        // TODO: Is there a way to check for a CONTIGUOUS-like datatype?
        // assert_eq!(param.as_ref().datatype, UCP_DATATYPE_CONTIG.into());
        let req_ptr = ucp_tag_send_nbx(self.ep, buf.as_ptr() as *const _,
                                       buf.len() * std::mem::size_of::<u8>(),
                                       tag, param.as_ref());
        Self::check_req_err(req_ptr)?;
        Ok(Request::from_raw(req_ptr, param))
    }

    /// Do a non-blocking tagged receive.
    pub unsafe fn tag_recv_nbx<'b>(
        &self,
        worker: Worker,
        buf: &'b mut [u8],
        tag: ucp_tag_t,
        param: RequestParam,
    ) -> Result<Request<'b>, ()> {
        println!("before ucp_tag_recv_nbx()");
        let req_ptr = ucp_tag_recv_nbx(worker.into_raw(),
                                       buf.as_mut_ptr() as *mut _,
                                       buf.len() * std::mem::size_of::<u8>(),
                                       tag, 0, param.as_ref());
        println!("after ucp_tag_recv_nbx()");
        Self::check_req_err(req_ptr)?;
        Ok(Request::from_raw(req_ptr, param))
    }

    /// Close the endpoint.
    pub unsafe fn close(self, worker: Worker, flags: u32) -> ucs_status_t {
        let param = RequestParam::default()
            .op_attr_mask(UCP_OP_ATTR_FIELD_FLAGS)
            .flags(flags);
        let close_req = ucp_ep_close_nbx(self.ep, param.as_ref());
        if rust_ucs_ptr_is_ptr(close_req) != 0 {
            let mut status = UCS_OK;
            loop {
                println!("Progress...");
                worker.progress();
                status = ucp_request_check_status(close_req);
                if status != UCS_INPROGRESS {
                    break;
                }
            }
            ucp_request_free(close_req);
            status
        } else {
            println!("Not a pointer...");
            rust_ucs_ptr_status(close_req)
        }
    }
}
