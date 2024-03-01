//! Message passing runtime and process management code.

pub struct Runtime;

impl Runtime {
    pub fn new() -> Runtime {
        Runtime
    }

    pub fn size(&self) -> u64 {
        1
    }

    pub fn id(&self) -> u64 {
        0
    }
}
