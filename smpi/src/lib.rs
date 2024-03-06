//! Safe MPI (SMPI) library.
use std::future::Future;
use smpi_base::Result;
use tokio::runtime::Builder;

mod communicator;
pub use communicator::Communicator;
mod p2p;

/// Initialize main function for SMPI application.
pub fn main<R, F, A>(f: F) -> R
where
    F: FnOnce(Communicator) -> A,
    A: Future<Output = R>
{
    // Initialize the tokio runtime
    let tokio_rt = Builder::new_multi_thread()
        .build()
        .unwrap();

    tokio_rt.block_on(async move {
        f(Communicator::new()).await
    })
}
