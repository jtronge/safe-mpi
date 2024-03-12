//! SMPI base data structures and traits.
use std::future::Future;
use std::pin::Pin;

mod buffer;
pub use buffer::{BufRead, BufWrite};

#[derive(Debug)]
pub enum Error {
    /// Feature not implemented
    NotImplemented,

    /// Could not reach the requested process
    Unreachable,

    /// Message transmission failed
    MessageTransmissionFailure,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Reachability enum indicating whether or not a certain rank is reachable
/// from this provider and how good the connection is.
#[derive(Debug)]
pub enum Reachability {
    /// Process is reachable with rough estimate of (latency, bandwidth)
    Reachable(u32, u32),
    /// Cannot send to this process
    Unreachable,
}

/// Point to point provider implementation
pub trait P2PProvider {
    /// Return the "reachability" for another process using this provider.
    fn reachability(&self, id: u64) -> Reachability;

    unsafe fn send_nb(
        &self,
        buf: *const u8,
        size: usize,
        type_id: u64,
        target: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>>;

    unsafe fn recv_nb(
        &self,
        buf: *mut u8,
        size: usize,
        type_id: u64,
        source: u64,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}
