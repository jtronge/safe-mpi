mod bincode;
pub use self::bincode::BincodeController;
mod messagepack;
pub use messagepack::MessagePackController;
mod postcard;
pub use self::postcard::PostcardController;
mod iovec;
pub use self::iovec::IovecController;
mod serde;
pub use self::serde::{SerdeController, SerdeScope};

use safe_mpi::{
    Result,
    RequestStatus,
};

pub trait Progress {
    type Request: Copy;

    fn progress(&mut self, req: Self::Request) -> Result<RequestStatus>;
}

/// Wait for all requests to complete for the scope.
pub fn wait_all<S>(scope: &mut S, reqs: &[S::Request]) -> Result<()>
where
    S: Progress,
{
    loop {
        let mut done = vec![false; reqs.len()];
        let mut not_done = 0;
        for (i, req) in reqs.iter().enumerate() {
            if done[i] {
                continue;
            }
            for _ in 0..16 {
                match scope.progress(*req)? {
                    RequestStatus::InProgress => not_done += 1,
                    RequestStatus::Complete => done[i] = true,
                }
            }
        }

        if not_done == 0 {
            break;
        }
    }
    Ok(())
}
