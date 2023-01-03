use ucx2_sys::{
    ucp_listener_h,
    UCS_OK,
    ucp_listener_create,
};
use std::mem::MaybeUninit;
use super::{Worker, ListenerParams};

pub struct Listener(ucp_listener_h);

impl Listener {
    pub fn new<'a>(worker: Worker, params: &ListenerParams<'a>) -> Listener {
        unsafe {
            let mut listener = MaybeUninit::<ucp_listener_h>::uninit();
            let status = ucp_listener_create(worker.into_raw(), params.as_ref(),
                                             listener.as_mut_ptr());
            if status != UCS_OK {
                panic!("ucp_listener_create() failed");
            }
            let listener = listener.assume_init();
            // TODO: query listening port
            Listener(listener)
        }
    }
}
