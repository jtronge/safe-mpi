//! UCX context handle
use std::cell::RefCell;
use std::default::Default;
use std::mem::MaybeUninit;
use std::rc::Rc;
// use log::{debug, info};
use crate::communicator::Communicator;
use crate::status_to_string;
use crate::Handle;
use smpi_ucx_sys::{
    ucp_ep_create, ucp_ep_h, ucp_ep_params_t, UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS, UCP_ERR_HANDLING_MODE_PEER, UCS_OK,
};

pub struct Context {
    /// Handle with ucx info
    handle: Rc<RefCell<Handle>>,
}

impl Context {
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Context {
        Context { handle }
    }

    /// Return the world communicator.
    #[allow(clippy::uninit_assumed_init)]
    pub fn world(&self) -> Communicator {
        unsafe {
            // Now create the single endpoint (this will change for multiple processes)
            let mut endpoint = MaybeUninit::<ucp_ep_h>::uninit();
            let params = ucp_ep_params_t {
                field_mask: (UCP_EP_PARAM_FIELD_REMOTE_ADDRESS
                             | UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE).into(),
                err_mode: UCP_ERR_HANDLING_MODE_PEER,
                address: self.handle.borrow().other_addr.as_ptr() as *const _,
                ..Default::default()
            };
            let status = ucp_ep_create(self.handle.borrow().worker, &params, endpoint.as_mut_ptr());
            if status != UCS_OK {
                panic!(
                    "Failed to create endpoint for worker: {}",
                    status_to_string(status)
                );
            }
            let endpoint = endpoint.assume_init();
            let _ = self.handle.borrow_mut().endpoint.insert(endpoint);
            Communicator::new(Rc::clone(&self.handle))
        }
    }
}
