use ucx2_sys::{
    ucp_worker_h,
    ucp_worker_create,
    ucp_worker_progress,
    ucp_worker_destroy,
    UCS_OK,
};
use std::mem::MaybeUninit;
use super::{Context, WorkerParams};

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

    pub unsafe fn destroy(self) {
        ucp_worker_destroy(self.into_raw());
    }
}
