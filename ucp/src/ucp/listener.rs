use ucx2_sys::{
    ucp_listener_h,
    UCS_OK,
    ucp_listener_create,
};
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use super::{Worker, ListenerParams};

#[repr(transparent)]
pub struct Listener<'a> {
    listener: ucp_listener_h,
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> Listener<'a> {
    pub fn new(worker: Worker, params: &ListenerParams<'a>) -> Listener<'a> {
        unsafe {
            let mut listener = MaybeUninit::<ucp_listener_h>::uninit();
            let status = ucp_listener_create(worker.into_raw(), params.as_ref(),
                                             listener.as_mut_ptr());
            if status != UCS_OK {
                panic!("ucp_listener_create() failed");
            }
            let listener = listener.assume_init();
            // TODO: query listening port
            Listener {
                listener,
                _phantom_data: PhantomData,
            }
        }
    }
}
