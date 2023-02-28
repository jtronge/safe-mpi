use std::net::{Ipv4Addr, SocketAddr};
use safe_mpi;
use clap::Parser;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IPv4 address of other process
    address: Ipv4Addr,
    /// TCP port of other process
    port: u16,
    /// Is this the server process?
    #[arg(short, long)]
    server: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestData {
    x: [f64; 2],
}

fn main() {
    let args = Args::parse();
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to init safe_mpi");
    let comm = sm.world();
    if args.server {
        let req = comm.irecv();
        let data: TestData = req.finish().unwrap();
        println!("data: {:?}", data);
    } else {
        let data = TestData {
            x: [1.3, 777.8],
        };
        println!("Sending data");
        let req = comm.isend(data);
        req.finish().unwrap();
    }
}
