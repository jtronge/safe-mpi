//! SMPI base data structures and traits.
use std::future::Future;

mod buffer;
pub use buffer::{BufRead, BufWrite};

#[derive(Debug)]
pub enum Error {
    NotImplemented,
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
pub trait Provider {
    /// Return the "reachability" for another process using this provider.
    fn reachability(&self, id: u64) -> Reachability;

    fn send_nb(
        &self,
        data: Box<dyn BufRead>,
        target: u64,
    ) -> Box<dyn Future<Output = Result<Box<dyn BufRead>>>>;

    fn recv_nb(
        &self,
        data: Box<dyn BufWrite>,
        source: u64,
    ) -> Box<dyn Future<Output = Result<Box<dyn BufWrite>>>>;
}
