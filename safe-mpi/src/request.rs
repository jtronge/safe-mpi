//! Request code.
use log::{debug, error, info};
use ucx2_sys::{
    rust_ucs_ptr_is_ptr,
    rust_ucs_ptr_status,
    rust_ucs_ptr_is_err,
    ucp_worker_progress,
    ucp_worker_h,
    UCS_INPROGRESS,
    UCS_OK,
};
use std::marker::PhantomData;
use std::os::raw::c_void;
use serde::{Serialize, de::DeserializeOwned};
use crate::status_to_string;

/// Request object.
///
/// TODO: this should have a lifetime to match the context object.
pub struct Request<T> {
    data: Option<T>,
    buf: Option<Vec<u8>>,
    done: Box<bool>,
    req: *mut c_void,
    worker: ucp_worker_h,
}

impl<T> Request<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(
        data: Option<T>,
        buf: Option<Vec<u8>>,
        done: Box<bool>,
        req: *mut c_void,
        worker: ucp_worker_h,
    ) -> Request<T> {
        Request {
            data,
            buf,
            done,
            req,
            worker,
        }
    }

    pub fn finish(mut self) -> Option<T> {
        assert!(self.req != std::ptr::null_mut());
        // TODO: Check for send/recv type
        unsafe {
            self.wait_loop();
        }
        self.data
    }

    unsafe fn wait_loop(&mut self) {
        if rust_ucs_ptr_is_ptr(self.req) == 0 {
            let status = rust_ucs_ptr_status(self.req);
            if status != UCS_OK {
                panic!("Request failed: {}", status_to_string(status));
            }
            // Already done
            return;
        }
        if rust_ucs_ptr_is_err(self.req) != 0 {
            panic!("Failed to send data");
        }

        let mut i = 0;
        loop {
            info!("In wait loop {}", i);
            // Make some progress
            for j in 0..1024 {
                ucp_worker_progress(self.worker);
            }
            // Then get the status
            let status = rust_ucs_ptr_status(self.req);
            debug!("status: {}", status_to_string(status));
            if status != UCS_INPROGRESS {
                // Request is finished
                if status != UCS_OK {
                    panic!(
                        "Request failed to complete: {}",
                        status_to_string(status),
                    );
                }
                break;
            }

            // Check if the done variable is set
            if *self.done {
                break;
            }
            i += 1;
        }
    }
}
