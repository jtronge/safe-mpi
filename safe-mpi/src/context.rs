use std::mem::MaybeUninit;
use std::rc::Rc;
use std::cell::RefCell;
// use log::{debug, info};
use ucx2_sys::{
    ucp_ep_create,
    ucp_ep_h,
    ucp_ep_params_t,
    UCP_ERR_HANDLING_MODE_PEER,
    UCP_EP_PARAM_FIELD_REMOTE_ADDRESS,
    UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE,
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
            let field_mask = UCP_EP_PARAM_FIELD_REMOTE_ADDRESS
                             | UCP_EP_PARAM_FIELD_ERR_HANDLING_MODE;
            params.field_mask = field_mask.into();
            params.err_mode = UCP_ERR_HANDLING_MODE_PEER;
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
