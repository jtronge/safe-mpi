use serde::{Serialize, de::DeserializeOwned};
// use log::{debug, error, info};
use ucx2_sys::{
    ucp_worker_h,
    ucp_ep_h,
};
// TODO: Replace with rmp_serde
use crate::context::Context;
use crate::request::{SendRequest, RecvRequest};

pub struct Communicator<'a> {
    /// Save a reference to the context
    context: &'a Context,
    /// One worker per communicator
    worker: ucp_worker_h,
    /// Endpoint to other process (in a multi-process scenario there would be
    /// multiple endpoints here)
    endpoint: ucp_ep_h,
}

impl<'a> Communicator<'a> {
    pub(crate) fn new(
        context: &'a Context,
        worker: ucp_worker_h,
        endpoint: ucp_ep_h,
    ) -> Communicator<'a> {
        Communicator {
            context,
            worker,
            endpoint,
        }
    }

    pub fn isend<T>(&self, data: T) -> SendRequest<T>
    where
        T: Serialize + DeserializeOwned,
    {
        SendRequest::new(data, self.worker, self.endpoint)
    }

    pub fn irecv<T>(&self) -> RecvRequest<T>
    where
        T: Serialize + DeserializeOwned,
    {
        RecvRequest::new(self.worker, self.endpoint)
    }
}
