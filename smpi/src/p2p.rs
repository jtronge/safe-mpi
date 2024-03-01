//! Point to point messaging base code and interface.
use std::future::Future;
use std::sync::{Arc, Mutex};
use smpi_base::{Result, Error, BufRead, BufWrite, Provider};
use smpi_runtime::Runtime;

/// Internal data structure for managing P2P calls.
pub(crate) struct Manager {
    runtime: Arc<Mutex<Runtime>>,
    providers: Vec<Box<dyn Provider>>,
}

impl Manager {
    pub(crate) fn new(runtime: Arc<Mutex<Runtime>>) -> Manager {
        Manager {
            runtime,
            providers: vec![],
        }
    }

    pub(crate) fn send_nb<T: BufRead>(
        &self,
        data: T,
        target: u64,
    ) -> impl Future<Output = Result<T>> {
        // assert!(target < self.size());
        async move {
            Err(Error::NotImplemented)
        }
    }

    /// Non-blocking receive a message from another process.
    pub(crate) fn recv_nb<T: BufWrite>(
        &self,
        data: Option<T>,
        source: u64,
    ) -> impl Future<Output = Result<T>> {
        // assert!(source < self.size());
        async move {
            Err(Error::NotImplemented)
        }
    }
}
