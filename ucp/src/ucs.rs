use ucx2_sys::{
    ucs_thread_mode_t,
    UCS_THREAD_MODE_SINGLE,
    UCS_THREAD_MODE_SERIALIZED,
    UCS_THREAD_MODE_MULTI,
    UCS_THREAD_MODE_LAST,
};
use std::result;
use crate::Status;

pub type Result<T> = result::Result<T, Status>;

pub struct ThreadMode;

impl ThreadMode {
    pub const SINGLE: ucs_thread_mode_t = UCS_THREAD_MODE_SINGLE;
    pub const SERIALIZED: ucs_thread_mode_t = UCS_THREAD_MODE_SERIALIZED;
    pub const MULTI: ucs_thread_mode_t = UCS_THREAD_MODE_MULTI;
    pub const LAST: ucs_thread_mode_t = UCS_THREAD_MODE_LAST;
}
