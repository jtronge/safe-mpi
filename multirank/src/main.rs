/// Test binary for multirank code.
use std::net::{IpAddr, SocketAddr};
use serde::Deserialize;
use safe_mpi;
use serde_yaml;

#[derive(Deserialize)]
struct Config {
    /// Current rank
    rank: u32,
    /// List of (IP, PORT) tuples.
    conns: Vec<(IpAddr, u16)>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        panic!("Requires one argument, the file path of the config");
    }
    let f = std::fs::File::open(&args[0]).unwrap();
    let cfg: Config = serde_yaml::from_reader(f).unwrap();
    let conns: Vec<SocketAddr> = cfg
        .conns
        .iter()
        .map(|conn| SocketAddr::from(*conn))
        .collect();
    let _ = safe_mpi::init_multirank(cfg.rank, &conns).unwrap();
}
