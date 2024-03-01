//! SMPI base data structures and traits.
use std::future::Future;

#[derive(Debug)]
pub enum Error {
    NotImplemented,
}

pub type Result<T> = std::result::Result<T, Error>;

// TODO: impl From for basic types with the below buffers

/// Trait for reading into a buffer (partially based on RSMPI's trait system).
pub unsafe trait BufRead {
    /// Return a buffer pointer and size in bytes.
    fn buffer(&self) -> (*const u8, usize);

    /// Return the type ID of the encoded type.
    fn type_id(&self) -> u64;
}

/// Trait for writing into a buffer (partially based on RSMPI's trait system).
pub unsafe trait BufWrite {
    /// Return the buffer pointer and size in bytes.
    fn buffer(&mut self) -> (*mut u8, usize);

    /// Return the type ID of the encdoed type.
    fn type_id(&self) -> u64;
}

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
