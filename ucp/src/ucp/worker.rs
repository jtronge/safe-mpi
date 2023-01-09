use ucx2_sys::{
    ucp_worker_h,
    ucp_worker_create,
    ucp_worker_progress,
    ucp_worker_destroy,
    ucp_am_handler_param_t,
    ucp_am_recv_param_t,
    ucs_status_t,
    UCS_OK,
    UCP_AM_HANDLER_PARAM_FIELD_ID,
    UCP_AM_HANDLER_PARAM_FIELD_CB,
    UCP_AM_HANDLER_PARAM_FIELD_ARG,
};
use std::mem::MaybeUninit;
use super::{Context, WorkerParams};
use std::os::raw::c_void;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Worker(ucp_worker_h);

impl Worker {
    pub fn new(context: Context, params: &WorkerParams) -> Worker {
        unsafe {
            let mut worker = MaybeUninit::<ucp_worker_h>::uninit();
            let status = ucp_worker_create(context.into_raw(), params.as_ref(),
                                           worker.as_mut_ptr());
            if status != UCS_OK {
                panic!("ucp_worker_create() failed");
            }
            let worker = worker.assume_init();
            Worker(worker)
        }
    }

    #[inline]
    pub fn into_raw(&self) -> ucp_worker_h {
        self.0
    }

    pub unsafe fn progress(&self) {
        ucp_worker_progress(self.into_raw());
    }

    pub unsafe fn register_am_handler<F>(&self, f: F)
    where
        F: Fn(usize) -> ucs_status_t,
    {
        let f: Box<dyn Fn(usize) -> ucs_status_t> = Box::new(f);
        let arg = Box::into_raw(Box::new(f)) as *mut c_void;
        let field_mask = UCP_AM_HANDLER_PARAM_FIELD_ID
                         | UCP_AM_HANDLER_PARAM_FIELD_CB
                         | UCP_AM_HANDLER_PARAM_FIELD_ARG;
        let param = ucp_am_handler_param_t {
            field_mask: field_mask.into(),
            id: 0,
            flags: 0,
            cb: Some(am_recv_callback),
            arg,
        };
    }

    pub unsafe fn destroy(self) {
        ucp_worker_destroy(self.into_raw());
    }
}

unsafe extern "C" fn am_recv_callback(
    arg: *mut c_void,
    header: *const c_void,
    header_length: usize,
    data: *mut c_void,
    length: usize,
    param: *const ucp_am_recv_param_t,
) -> ucs_status_t {
    let cb = arg as *mut Box<dyn Fn(usize) -> ucs_status_t>;
    (*cb)(length)
}
