//! Safe MPI (SMPI) library.
use std::future::Future;
use futures::executor;

pub enum Error {
    NotImplemented,
}

type Result<T> = std::result::Result<T, Error>;

pub trait Communicator {
    /// Return number of members in the process group.
    fn size(&self) -> u64 { 1 }

    /// Return the ID of this process.
    fn id(&self) -> u64 { 0 }
}

pub trait PointToPoint: Communicator {
    /// Non-blocking send to another process.
    ///
    /// This method takes ownership of the data buffer and returns it on
    /// completion.
    fn send_nb<T: BufRead>(
        &self,
        data: T,
        target: u64,
    ) -> impl Future<Output = Result<T>> {
        assert!(target < self.size());
        async move {
            Err(Error::NotImplemented)
        }
    }

    /// Send a message to another process (blocking).
    ///
    /// Takes ownership of data buffer and returns it on completion.
    fn send<T: BufRead>(&self, data: T, target: u64) -> Result<T> {
        executor::block_on(self.send_nb(data, target))
    }

    /// Non-blocking receive a message from another process.
    fn recv_nb<T: BufWrite>(
        &self,
        data: Option<T>,
        source: u64,
    ) -> impl Future<Output = Result<T>> {
        assert!(source < self.size());
        async move {
            Err(Error::NotImplemented)
        }
    }

    /// Receive a message from another process (blocking).
    fn recv<T: BufWrite>(&self, data: Option<T>, source: u64) -> Result<T> {
        executor::block_on(self.recv_nb(data, source))
    }
}

/// Trait for reading into a buffer (partially based on RSMPI's trait system).
pub trait BufRead {
    fn ptr(&self) -> *const u8;
    fn count(&self) -> usize;
}

/// Trait for writing into a buffer (partially based on RSMPI's trait system).
pub trait BufWrite {
    fn allocate(count: usize) -> Self;
    fn count(&self) -> usize;
    fn ptr_mut(&mut self) -> *mut u8;
}
