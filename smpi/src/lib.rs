//! Safe MPI (SMPI) library.

pub enum Error {}

type Result<T> = std::result::Result<T, Error>;

struct Communicator;

impl Communicator {
    /// Return number of members in the process group.
    pub fn size(&self) -> u64 { 1 }

    /// Return the ID of this process.
    pub fn id(&self) -> u64 { 0 }

    /// Send a message to another process.
    pub fn send<T: Message>(&self, message: &T, target: u64) -> Result<()> {
        Ok(())
    }

    /// Receive a message from another process.
    pub fn recv<T: Message>(&self, message: &mut T, source: u64) -> Result<()> {
        Ok(())
    }
}

pub trait Message {}
