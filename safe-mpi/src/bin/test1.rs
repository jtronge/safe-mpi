use std::net::{Ipv4Addr, SocketAddr};
use safe_mpi;
use clap::Parser;

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

fn main() {
/*
    let mut args = env::args();
    args.next();
    // Get the other address
    let address = args.next().unwrap();
    let send = &args.next().unwrap() == "send";
*/
    let args = Args::parse();
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to init safe_mpi");
    let comm = sm.world();
    if args.server {
        let mut buf = [0; 4];
        comm.recv(&mut buf);
        // comm.stream_recv(&mut buf);
        println!("data: {:?}", buf);
    } else {
        println!("Sending data");
        comm.send(&[1, 2, 3, 4]);
        // comm.stream_send(&[1, 2, 3, 4]);
    }
}
