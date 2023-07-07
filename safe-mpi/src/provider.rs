use crate::{Tag, Rank};

#[derive(Debug)]
pub enum ProviderError {
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

pub type ProviderResult<T> = std::result::Result<T, ProviderError>;

/// Header trait
///
/// WARNING: This trait is considered unsafe since it should only be
/// implemented on types that can be safely cast to a pointer and sent over a
/// network. This type should be self-contained, without any references to other
/// data on this process.
pub unsafe trait Header {
    /// Unique ID for this header type - helps ensure type matching of header
    fn id() -> u32;
}

pub trait ProviderRequest<'scope> {}

/// Provider non-blocking scope interface.
///
/// TODO: I don't quite understand the need for the 'env -- it must be some
/// complexity of the type system.
pub trait ProviderNonBlockingScope<'scope, 'env> {
    type Request: ProviderRequest<'scope>;

    /// Non-blocking immediate send.
    fn isend<H>(
        &mut self,
        rank: Rank,
        tag: Tag,
        header: &'scope H,
        data: &'scope [u8],
    ) -> ProviderResult<Self::Request>
    where
        H: Header;

    /// Non-blocking immediate receive.
    fn irecv<H>(
        &mut self,
        rank: Rank,
        tag: Tag,
        header: &'scope mut H,
        data: &'scope mut [u8],
    ) -> ProviderResult<Self::Request>
    where
        H: Header;
}

/// A provider that can act as a communicator.
///
/// TODO: Need to think about global state and how that relates to calling
/// progress functions
pub trait Provider {
    /// Scope associated type to be used for non blocking calls
    ///
    /// TODO: Not sure if this higher-ranked type bound (with for<'a>) is doing
    /// what I want it to do for the scope below.
    type NonBlockingScope: for<'a> ProviderNonBlockingScope<'a, 'a>;

    /// Return the "reachability" for a process.
    fn reachability(&self, rank: Rank) -> Reachability;

    /// Blocking send.
    fn send<H>(
        &mut self,
        rank: Rank,
        tag: Tag,
        header: &H,
        data: &[u8],
    ) -> ProviderResult<()>
    where
        H: Header;

    /// Blocking receive into the header and data buffer.
    fn recv_into<H>(
        &mut self,
        rank: Rank,
        tag: Tag,
        header: &mut H,
        data: &mut [u8],
    ) -> ProviderResult<()>
    where
        H: Header;

    /// Provide a scope for doing safe non-blocking calls that borrow memory.
    fn non_blocking<'env, F, R>(&mut self, f: F) -> R
    where
        F: for<'scope> FnOnce(&'scope mut Self::NonBlockingScope) -> R;
}
