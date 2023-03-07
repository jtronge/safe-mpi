use std::io::Write;
use std::mem::MaybeUninit;
use std::net::{SocketAddr, TcpStream, TcpListener, Shutdown};
use std::rc::Rc;
use std::cell::RefCell;
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
use crate::Handle;
use crate::communicator::Communicator;
use crate::status_to_string;

pub struct Context {
    /// Handle with ucx info
    handle: Rc<RefCell<Handle>>,
}

impl Context {
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Context {
        Context {
            handle,
        }
    }

    /// Return the world communicator.
    pub fn world(&self) -> Communicator {
        unsafe {
            // Now create the single endpoint (this will change for multiple processes)
            let mut endpoint = MaybeUninit::<ucp_ep_h>::uninit();
            let mut params = MaybeUninit::<ucp_ep_params_t>::uninit().assume_init();
            params.field_mask = UCP_EP_PARAM_FIELD_REMOTE_ADDRESS.into();
            params.address = self.handle.borrow().other_addr.as_ptr() as *const _;
            let status = ucp_ep_create(self.handle.borrow().worker, &params, endpoint.as_mut_ptr());
            if status != UCS_OK {
                panic!("Failed to create endpoint for worker: {}", status_to_string(status));
            }
            let endpoint = endpoint.assume_init();
            let _ = self.handle.borrow_mut().endpoint.insert(endpoint);
            Communicator::new(Rc::clone(&self.handle))
        }
    }
}
