//! Communicator with external application-level API.
use std::sync::{Arc, Mutex};
use std::future::Future;
use futures::executor;
use smpi_base::{Result, Error, BufRead, BufWrite};
use smpi_runtime::Runtime;
use crate::p2p;

pub struct Communicator {
    runtime: Arc<Mutex<Runtime>>,
    p2p_manager: p2p::Manager,
}

impl Communicator {
    /// Internal initialization function.
    pub(crate) fn new() -> Communicator {
        let runtime = Runtime::new();
        let runtime = Arc::new(Mutex::new(Runtime));
        let p2p_manager = p2p::Manager::new(Arc::clone(&runtime));
        Communicator {
            runtime,
            p2p_manager,
        }
    }

    /// Return number of members in the process group.
    pub fn size(&self) -> u64 {
        self.runtime.lock().unwrap().size()
    }

    /// Return the ID of this process.
    pub fn id(&self) -> u64 {
        self.runtime.lock().unwrap().id()
    }

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
