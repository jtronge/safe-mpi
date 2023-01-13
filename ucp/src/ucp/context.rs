use ucx2_sys::{
    ucp_context_h,
    ucp_params_t,
    rust_ucp_init,
    UCP_PARAM_FIELD_FEATURES,
    ucp_feature,
    UCS_OK,
    ucp_cleanup,
};
use std::mem::MaybeUninit;
use crate::{
    Feature,
    ucs,
    Status,
};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct Context(ucp_context_h);

impl Context {
    pub fn new(features: ucp_feature) -> ucs::Result<Context> {
        unsafe {
            // The docs declare this to be UB, but this seems to be how the C API works
            let mut context = MaybeUninit::<ucp_context_h>::uninit();

            let mut params: ucp_params_t = unsafe { MaybeUninit::zeroed().assume_init() };
            params.field_mask = UCP_PARAM_FIELD_FEATURES.into();
            // params.features = UCP_FEATURE_TAG.into();
            params.features = features.into();
    
            let status = rust_ucp_init(&params, std::ptr::null(),
                                       context.as_mut_ptr());
            if status != UCS_OK {
                return Err(Status::from_raw(status));
            }
            let context = context.assume_init();
            Ok(Context(context))
        }
    }

    #[inline]
    pub fn into_raw(&self) -> ucp_context_h {
        self.0
    }

    pub unsafe fn cleanup(self) {
        ucp_cleanup(self.into_raw());
    }
}
