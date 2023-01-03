use ucx2_sys::{
    ucp_context_h,
    ucp_params_t,
    rust_ucp_init,
    UCP_PARAM_FIELD_FEATURES,
    UCP_FEATURE_AM,
    UCS_OK,
    ucp_cleanup,
};
use std::mem::MaybeUninit;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Context(ucp_context_h);

impl Context {
    pub fn new() -> Context {
        // The docs declare this to be UB, but this seems to be how the C API works
        let mut context = MaybeUninit::<ucp_context_h>::uninit();
    
        let mut params: ucp_params_t = unsafe { MaybeUninit::zeroed().assume_init() };
        params.field_mask = UCP_PARAM_FIELD_FEATURES.into();
        params.features = UCP_FEATURE_AM.into();
    
        let status = unsafe {
            rust_ucp_init(&params, std::ptr::null(), context.as_mut_ptr())
        };
        if status != UCS_OK {
            panic!("ucp_init() failed");
        }
        let context = unsafe { context.assume_init() };
        Context(context)
    }

    #[inline]
    pub fn into_raw(&self) -> ucp_context_h {
        self.0
    }

    pub unsafe fn cleanup(self) {
        ucp_cleanup(self.into_raw());
    }
}
