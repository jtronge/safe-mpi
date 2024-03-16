//! Data structures for communication with the runtime engine.
//!
//! The runtime engine manages process startup and shutdown as well as
//! communication of process and hardware-specification information.
use serde::{Serialize, Deserialize};

/// Request keys, representing various properties that can be obtained from the
/// runtime engine, and sent to the runtime engine.
#[derive(Serialize, Deserialize)]
pub enum Key {
    /// Return the processes on the local node
    ProcessesOnNode,
}

/// Request made by a process to the runtime engine.
#[derive(Serialize, Deserialize)]
pub struct Request {
    /// Request key
    key: Key,
}
