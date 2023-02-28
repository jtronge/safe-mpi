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
/*
    pub fn send(&self, buf: &[u8]) {
        unsafe {
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            let req = ucp_tag_send_nbx(
                self.endpoint,
                buf.as_ptr() as *const _,
                buf.len() * std::mem::size_of::<u8>(),
                0,
                &param,
            );

            wait_loop(self.worker, req, &false);
        }
    }

    pub fn recv(&self, buf: &mut [u8]) {
        unsafe {
            let mut done = false;
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK
                                 | UCP_OP_ATTR_FIELD_USER_DATA;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            param.cb.recv = Some(tag_recv_nbx_callback);
            // XXX: This is not following proper safety protocol
            /*
            let done_ptr = &mut done as *mut _;
            param.user_data = done_ptr;
            */
            let req = ucp_tag_recv_nbx(
                self.worker,
                buf.as_mut_ptr() as *mut _,
                buf.len() * std::mem::size_of::<u8>(),
                0,
                0,
                &param,
            );

            wait_loop(self.worker, req, &false);
        }
    }
*/

    pub fn isend<T>(&self, data: T) -> SendRequest<T>
    where
        T: Serialize + DeserializeOwned,
    {
/*
            let buf: Vec<u8> = bincode::serialize(&data)
                .expect("Failed to serialize data");
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            let req = ucp_tag_send_nbx(
                self.endpoint,
                buf.as_ptr() as *const _,
                buf.len() * std::mem::size_of::<u8>(),
                0,
                &param,
            );

            Request::new(Some(data), Some(buf), Box::new(false), req, self.worker)
*/
        SendRequest::new(data, self.worker, self.endpoint)
    }

    pub fn irecv<T>(&self) -> RecvRequest<T>
    where
        T: Serialize + DeserializeOwned,
    {
        RecvRequest::new(self.worker, self.endpoint)
    }

/*
    /// Do a streaming send.
    pub fn stream_send(&self, buf: &[u8]) {
        unsafe {
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            param.cb.send = Some(send_nbx_callback);
            let req = ucp_stream_send_nbx(
                self.endpoint,
                buf.as_ptr() as *const _,
                buf.len() * std::mem::size_of::<u8>(),
                &param,
            );
            wait_loop(self.worker, req, &false);
        }
    }

    pub fn stream_recv(&self, buf: &mut [u8]) {
        unsafe {
            let mut length = 0;
            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_CALLBACK;
            param.datatype = rust_ucp_dt_make_contig(buf.len()).try_into().unwrap();
            param.cb.recv_stream = Some(stream_recv_nbx_callback);
            let req = ucp_stream_recv_nbx(
                self.endpoint,
                buf.as_ptr() as *mut _,
                buf.len() * std::mem::size_of::<u8>(),
                &mut length,
                &param,
            );
            if req == std::ptr::null_mut() {
                // TODO: What to do with the length?
            }
            wait_loop(self.worker, req, &false);
        }
    }
*/
}
