//! UCX Status wrapping code
use std::fmt;
use std::ffi::CStr;
use ucx2_sys::{
    ucs_status_t,
    ucs_status_string,
    UCS_OK,
};

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq)]
pub struct Status(ucs_status_t);

impl fmt::Debug for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status [\"{}\"]", self.to_string())
    }
}

impl Status {
    pub const OK: Status = Status(UCS_OK);

    /// Create a status from the raw data type.
    #[inline]
    pub fn from_raw(status: ucs_status_t) -> Status {
        Status(status)
    }

    /// Convert the status to a String.
    pub fn to_string(&self) -> String {
        unsafe {
            let ptr = ucs_status_string(self.0);
            CStr::from_ptr(ptr)
                .to_str()
                .expect("Could not convert pointer from ucs_status_string() into a Rust str")
                .to_string()
        }
    }
}
