use ucx2_sys::{
    ucp_ep_h,
    ucp_ep_create,
    ucp_tag_t,
    UCS_OK,
};
use std::mem::MaybeUninit;
use super::{Worker, Request, RequestParam, EndpointParams};

pub struct Endpoint(ucp_ep_h);

impl Endpoint {
    pub fn new(worker: Worker, params: &EndpointParams) -> Endpoint {
        let mut ep = MaybeUninit::<ucp_ep_h>::uninit();
        let status = unsafe {
            ucp_ep_create(worker.into_raw(), params.as_ref(), ep.as_mut_ptr())
        };
        if status != UCS_OK {
            panic!("ucp_ep_create() failed");
        }
        let ep = unsafe { ep.assume_init() };
        Endpoint(ep)
    }

    /// Do a non-blocking tagged send.
    pub unsafe fn tag_send_nbx<'a>(
        &self,
        buf: &'a [u8],
        tag: ucp_tag_t,
        param: &RequestParam,
    ) -> Result<Request<'a>, ()> {
        // TODO
        Err(())
    }

    /// Return a non-blocking request that can be used to close the endpoint.
    pub unsafe fn close_nbx<'a>(
        self,
        param: &'a RequestParam,
    ) -> Result<Request<'a>, ()> {
        // TODO
        Err(())
    }
}
