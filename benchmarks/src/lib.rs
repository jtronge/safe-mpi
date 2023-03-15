use std::net::Ipv4Addr;
use serde::de::DeserializeOwned;
use std::path::Path;
use clap::{
    Parser,
    ValueEnum,
};

pub mod data_controllers;
mod latency;
pub use latency::{
    latency,
    LatencyOptions,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// IPv4 address of other process
    pub address: Ipv4Addr,
    /// TCP port of other process
    pub port: u16,
    /// Is this the server process?
    #[arg(short, long)]
    pub server: bool,
    /// Which kind of benchmark to run
    #[arg(value_enum)]
    pub kind: Kind,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Kind {
    MessagePack,
    Postcard,
    Bincode,
}

#[derive(Copy, Clone, Debug)]
pub enum BenchmarkError {
    IOError,
    DeserializeError,
}

/// Load a config from a file path.
pub fn load_config<P, T>(path: P) -> Result<T, BenchmarkError>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    serde_yaml::from_reader(
        std::fs::File::open(path).map_err(|_| BenchmarkError::IOError)?
    ).map_err(|_| BenchmarkError::DeserializeError)
}
