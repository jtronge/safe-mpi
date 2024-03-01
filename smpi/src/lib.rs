//! Safe MPI (SMPI) library.
use smpi_base::Result;

mod communicator;
pub use communicator::Communicator;
mod p2p;

/// Initialize main function for SMPI application.
pub fn main<R, F>(f: F) -> R
where
    F: FnOnce(&mut Communicator) -> R,
{
    let mut comm = Communicator::new();
    f(&mut comm)
}
