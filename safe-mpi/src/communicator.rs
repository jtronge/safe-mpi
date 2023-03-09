use std::rc::Rc;
use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use serde::{Serialize, de::DeserializeOwned};
// use log::{debug, error, info};
use ucx2_sys::{
    rust_ucp_dt_make_contig,
    ucs_status_t,
    ucp_worker_h,
    ucp_worker_progress,
    ucp_worker_wait,
    ucp_ep_close_nb,
    ucp_ep_h,
    ucp_request_free,
    ucp_tag_msg_recv_nbx,
    ucp_request_param_t,
    ucp_tag_recv_info_t,
    ucp_tag_recv_nbx,
    ucp_tag_probe_nb,
    ucp_tag_send_nbx,
    ucp_tag_t,
    UCP_EP_CLOSE_MODE_FLUSH,
    UCP_OP_ATTR_FIELD_DATATYPE,
    UCP_OP_ATTR_FIELD_CALLBACK,
    UCP_OP_ATTR_FIELD_USER_DATA,
    UCP_OP_ATTR_FLAG_NO_IMM_CMPL,
    UCS_OK,
};
use rmp_serde;
// TODO: Replace with rmp_serde
use crate::{
    Result,
    Error,
    Handle,
    status_to_string,
};
use crate::context::Context;
// use crate::request::{SendRequest, RecvRequest};
use crate::util::wait_loop;
use crate::callbacks::{
    send_nbx_callback,
    tag_recv_nbx_callback,
};

pub struct Communicator {
    handle: Rc<RefCell<Handle>>,
}

pub fn send(worker: ucp_worker_h, endpoint: ucp_ep_h, tag: ucp_tag_t, buf: &[u8]) -> Result<usize> {
    unsafe {
        let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
        param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                             | UCP_OP_ATTR_FIELD_CALLBACK
                             | UCP_OP_ATTR_FIELD_USER_DATA;
        param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
        param.cb.send = Some(send_nbx_callback);
        // Callback info
        let cb_info: *mut bool = Box::into_raw(Box::new(false));
        param.user_data = cb_info as *mut _;

        let req = ucp_tag_send_nbx(
            endpoint,
            buf.as_ptr() as *const _,
            buf.len(),
            tag,
            &param,
        );
        wait_loop(worker, req, || *cb_info).unwrap();

        let _ = Box::from_raw(cb_info);
        Ok(buf.len())
    }
}


pub fn recv(worker: ucp_worker_h, tag: ucp_tag_t, buf: &mut [u8]) -> Result<usize> {
    unsafe {
        let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
        param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE
                             | UCP_OP_ATTR_FIELD_CALLBACK
                             | UCP_OP_ATTR_FIELD_USER_DATA;
        param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
        param.cb.recv = Some(tag_recv_nbx_callback);
        // Callback info
        let cb_info: *mut bool = Box::into_raw(Box::new(false));
        param.user_data = cb_info as *mut _;
        let req = ucp_tag_recv_nbx(
            worker,
            buf.as_mut_ptr() as *mut _,
            buf.len(),
            tag,
            0,
            &param,
        );
        wait_loop(worker, req, || *cb_info).unwrap();
        let _ = Box::from_raw(cb_info);
        Ok(buf.len())
    }
}

impl Communicator {
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> Communicator {
        Communicator {
            handle,
        }
    }

    pub fn send<T>(&self, data: &T) -> Result<usize>
    where
        T: Serialize + DeserializeOwned,
    {
        let buf = rmp_serde::to_vec(data).unwrap();
        let worker = self.handle.borrow().worker;
        let endpoint = self.handle.borrow().endpoint.unwrap();
        send(worker, endpoint, 0, &buf)
    }

    pub fn recv<T>(&self) -> Result<T>
    where
        T: Serialize + DeserializeOwned + Default,
    {
        unsafe {
            let mut info = MaybeUninit::<ucp_tag_recv_info_t>::uninit();
            let worker = self.handle.borrow().worker;
            let mut msg;
            loop {
                // Make sure to call ucp_worker_progress first, otherwise bad
                // things will happen
                ucp_worker_progress(worker);
                msg = ucp_tag_probe_nb(worker, 0, 0, 1, info.as_mut_ptr());
                if msg != std::ptr::null_mut() {
                    break;
                }
            }
            let info = info.assume_init();
            let mut buf = vec![0; info.length];

            let mut param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_CALLBACK
                                 | UCP_OP_ATTR_FIELD_DATATYPE
                                 | UCP_OP_ATTR_FIELD_USER_DATA
                                 | UCP_OP_ATTR_FLAG_NO_IMM_CMPL;
            param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
            param.cb.recv = Some(tag_recv_nbx_callback);
            let cb_info = Box::into_raw(Box::new(false));
            param.user_data = cb_info as *mut _;
            let req = ucp_tag_msg_recv_nbx(worker, buf.as_mut_ptr() as *mut _, info.length, msg, &param);
            wait_loop(worker, req, || *cb_info).unwrap();
            let _ = Box::from_raw(cb_info);
            rmp_serde::decode::from_slice(&buf)
                .map_err(|_| Error::DeserializeError)
        }
/*
        let ulen = std::mem::size_of::<usize>();
        let mut ubuf = vec![0; ulen];
        recv(worker, 1, &mut ubuf).unwrap();
        let len = usize::from_be_bytes(ubuf.try_into().unwrap());
        let mut buf = vec![0; len];
        recv(worker, 0, &mut buf).unwrap();
        rmp_serde::decode::from_slice(&buf)
            .map_err(|err| Error::DeserializeError)
*/
/*
            Ok(T::default())
*/
/*
            let worker = self.handle.borrow().worker;
            let mut info = MaybeUninit::<ucp_tag_recv_info_t>::uninit().assume_init();
            let mut msg = std::ptr::null_mut();
            loop {
                msg = ucp_tag_probe_nb(
                    worker,
                    0,
                    0,
                    1, // remove this request from the library
                    &mut info,
                );
                if msg == std::ptr::null_mut() {
                    println!("probed nothing...");
                    if ucp_worker_progress(worker) == 0 {
                        println!("Waiting on worker");
                        let status = ucp_worker_wait(worker);
                        if status != UCS_OK {
                            println!("worker wait error: {}", status_to_string(status));
                            return Err(Error::WorkerWait(status));
                        }
                    }
                    continue;
                }
            }
            let mut buf = vec![];
            buf.resize(info.length, 0u8);
            let param = MaybeUninit::<ucp_request_param_t>::uninit().assume_init();
            param.op_attr_mask = UCP_OP_ATTR_FIELD_DATATYPE | UCP_OP_ATTR_FIELD_CALLBACK | UCP_OP_ATTR_FIELD_USER_DATA;
            param.datatype = rust_ucp_dt_make_contig(1).try_into().unwrap();
            param.cb.recv = Some(tag_recv_nbx_callback);
            let cb_info: *mut bool = Box::into_raw(Box::new(false));
            param.user_data = cb_info as *mut _;
            // Receive the probed message
            let req = ucp_tag_msg_recv_nbx(
                worker,
                buf.as_mut_ptr() as *mut _,
                buf.len(),
                msg,
                &param,
            );
            wait_loop(worker, req, || *cb_info).unwrap();
            // Deallocate the callback info
            let _ = Box::from_raw(cb_info);
        }
*/
    }

/*
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
*/
}

pub(crate) unsafe extern "C" fn tag_recv_info_callback(
    req: *mut c_void,
    status: ucs_status_t,
    info: *mut ucp_tag_recv_info_t,
) {
}
