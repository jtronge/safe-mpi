pub use ucx2_sys::*;
pub mod ucp;
// TODO: Don't expose ucs_status_t, instead use a wrapping Status struct.
mod status;
pub use status::Status;

pub struct Feature;

impl Feature {
    pub const TAG: ucp_feature = UCP_FEATURE_TAG;
    pub const STREAM: ucp_feature = UCP_FEATURE_STREAM;
}

pub mod ucs;
pub use ucs::Result;
