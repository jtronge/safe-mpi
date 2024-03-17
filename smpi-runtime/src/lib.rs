//! Message passing runtime and process management code.
use std::io::prelude::*;
use std::net::TcpStream;
use std::env;
use serde::de::DeserializeOwned;
use smpi_base::runtime::Key;

struct RuntimeEngine {
    stream: TcpStream,
}

impl RuntimeEngine {
    fn new(stream: TcpStream) -> RuntimeEngine {
        RuntimeEngine {
            stream,
        }
    }

    /// Retrieve the value for a key from the backend runtime engine.
    fn get<T: DeserializeOwned>(&self, key: Key) -> Option<T> {
        // TODO
        None
    }
}

pub struct Runtime {
    engine: RuntimeEngine,
}

impl Runtime {
    pub fn new() -> Runtime {
        let runtime_addr = env::var("SMPI_RUNTIME_ADDR")
            .expect("failed to retrieve SMPI runtime address");
        let stream = TcpStream::connect(&runtime_addr)
            .expect("failed to connect to runtime");
        Runtime {
            engine: RuntimeEngine::new(stream),
        }
    }

    /// Return the total number of processes.
    pub fn size(&self) -> u64 {
        1
    }

    /// Return the unique ID for this process among all processes.
    pub fn id(&self) -> u64 {
        0
    }

    /// Get the unique ID for this node.
    pub fn node_id(&self) -> u64 {
        0
    }

    /// Return an iterator over the IDs of all processes that are on the given
    /// node.
    pub fn node_process_ids(&self, node_id: u64) -> Vec<u64> {
        self.engine
            .get(Key::ProcessesOnNode(node_id))
            .expect("failed to retrieve node process IDs")
    }
}
