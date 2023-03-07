use std::rc::Rc;
use std::cell::RefCell;
use serde::{Serialize, de::DeserializeOwned};
// use log::{debug, error, info};
use ucx2_sys::{
    ucp_worker_h,
    ucp_ep_close_nb,
    ucp_ep_h,
    ucp_request_free,
    UCP_EP_CLOSE_MODE_FLUSH,
};
// TODO: Replace with rmp_serde
use crate::Handle;
use crate::context::Context;
use crate::request::{SendRequest, RecvRequest};
use crate::util::wait_loop;

pub struct Communicator {
    handle: Rc<RefCell<Handle>>,
}

impl Communicator {
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Communicator {
        Communicator {
            handle,
        }
    }

    pub fn isend<T>(&self, data: T) -> SendRequest<T>
    where
        T: Serialize + DeserializeOwned,
    {
        SendRequest::new(data, Rc::clone(&self.handle))
    }

    pub fn irecv<T>(&self) -> RecvRequest<T>
    where
        T: Serialize + DeserializeOwned,
    {
        RecvRequest::new(Rc::clone(&self.handle))
    }
}
