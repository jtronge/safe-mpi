use std::io::Write;
use std::mem::MaybeUninit;
use std::net::{SocketAddr, TcpStream, TcpListener, Shutdown};
use log::{debug, info};
use ucx2_sys::{
    ucp_cleanup,
    ucp_context_h,
    ucp_worker_h,
    ucp_worker_create,
    ucp_worker_destroy,
    ucp_worker_get_address,
    ucp_worker_release_address,
    ucp_worker_params_t,
    ucp_ep_create,
    ucp_ep_h,
    ucp_ep_params_t,
    ucp_address_t,
    UCP_WORKER_PARAM_FIELD_THREAD_MODE,
    UCS_THREAD_MODE_SINGLE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCS_OK,
};
use crate::communicator::Communicator;
use crate::status_to_string;

pub struct Context {
    /// UCP context
    context: ucp_context_h,
    /// Main worker
    worker: ucp_worker_h,
    /// Address of other process
    other_addr: Vec<u8>
}

impl Context {
    pub(crate) fn new(
        context: ucp_context_h,
        worker: ucp_worker_h,
        other_addr: Vec<u8>,
    ) -> Context {
        Context {
            context,
            worker,
            other_addr,
        }
    }

    /// Return the world communicator.
    pub fn world<'a>(&'a self) -> Communicator<'a> {
        unsafe {
            // Now create the single endpoint (this will change for multiple processes)
            let mut endpoint = MaybeUninit::<ucp_ep_h>::uninit();
            let mut params = MaybeUninit::<ucp_ep_params_t>::uninit().assume_init();
            params.field_mask = UCP_EP_PARAM_FIELD_REMOTE_ADDRESS.into();
            params.address = self.other_addr.as_ptr() as *const _;
            let status = ucp_ep_create(self.worker, &params, endpoint.as_mut_ptr());
            if status != UCS_OK {
                panic!("Failed to create endpoint for worker: {}", status_to_string(status));
            }
            let endpoint = endpoint.assume_init();

            Communicator::new(self, self.worker, endpoint)
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            ucp_cleanup(self.context);
            ucp_worker_destroy(self.worker);
        }
    }
}
