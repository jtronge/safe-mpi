use clap::{Parser, ValueEnum};
use serde::de::DeserializeOwned;
use std::net::Ipv4Addr;
use std::path::Path;

mod latency;
pub use latency::{latency, LatencyOptions};
mod bw;
pub use bw::{bw, BandwidthOptions};
