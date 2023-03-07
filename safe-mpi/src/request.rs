//! Request code.
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
// use log::{debug, info};
use ucx2_sys::{
    ucp_worker_h,
    ucp_ep_h,
};
use serde::{Serialize, de::DeserializeOwned};
use crate::Handle;
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
    pub(crate) fn new(handle: Rc<RefCell<Handle>>) -> RecvRequest<T> {
        RecvRequest {
            stream: Stream::new(handle),
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
    pub(crate) fn new(
        data: T,
        handle: Rc<RefCell<Handle>>,
    ) -> SendRequest<T> {
        SendRequest {
            data,
            stream: Stream::new(handle),
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
}
