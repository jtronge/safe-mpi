use std::net::Ipv4Addr;
use clap::Parser;

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
}

