//! Request code.
use std::io::Write;
// use log::{debug, info};
use ucx2_sys::{
    ucp_worker_h,
    ucp_ep_h,
};
use serde::{Serialize, de::DeserializeOwned};
use crate::stream::Stream;
use std::marker::PhantomData;
use rmp_serde::{
    self,
    encode,
    decode,
};

pub struct RecvRequest<T> {
    stream: Stream,
    _marker: PhantomData<T>,
}

impl<T> RecvRequest<T>
where
    T: Serialize + DeserializeOwned,
{
    pub(crate) fn new(worker: ucp_worker_h, endpoint: ucp_ep_h) -> RecvRequest<T> {
        RecvRequest {
            stream: Stream::new(worker, endpoint),
            _marker: PhantomData,
        }
    }

    pub fn finish(mut self) -> Result<T, decode::Error> {
        rmp_serde::from_read(&mut self.stream)
    }
}

#[derive(Debug)]
pub enum SendError {
    EncodeError(encode::Error),
    IOError(std::io::Error),
}

/// Send request object.
///
/// TODO: this should have a lifetime to match the context object.
pub struct SendRequest<T> {
    data: T,
    stream: Stream,
}

impl<T> SendRequest<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(
        data: T,
        worker: ucp_worker_h,
        endpoint: ucp_ep_h,
    ) -> SendRequest<T> {
        SendRequest {
            data,
            stream: Stream::new(worker, endpoint),
        }
    }

    pub fn finish(mut self) -> Result<T, SendError> {
        // assert!(self.req != std::ptr::null_mut());
        // TODO: Check for send/recv type
        //unsafe {
        //    self.wait_loop();
        //}
        // self.data
        encode::write(&mut self.stream, &self.data)
            .map_err(|err| SendError::EncodeError(err))?;
        self.stream.flush()
            .map_err(|err| SendError::IOError(err))?;
        Ok(self.data)
    }

/*
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
            for _ in 0..1024 {
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
*/
}
