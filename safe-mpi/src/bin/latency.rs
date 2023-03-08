use std::net::SocketAddr;
use clap::Parser;
use safe_mpi::{
    self,
    Args,
};
// use nalgebra::Matrix3;

const ITERATIONS: usize = 512;
const SKIP: usize = 16;
const WARMUP_VALIDATION: usize = 8;

fn main() {
    let args = Args::parse();
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to initialize safe_mpi");
    let comm = sm.world();

    let mut count = 2;
    while count <= 512 {
        println!("Count: {}", count);
        let s_buf: Vec<f64> = (0..count).map(|i| i as f64).collect();
        for i in 0..ITERATIONS + SKIP {
            if args.server {
                for j in 0..=WARMUP_VALIDATION {
                    comm.send(&s_buf).unwrap();
                    let _data: Vec<f64> = comm.recv().unwrap();
                }
            } else {
                for j in 0..=WARMUP_VALIDATION {
                    let _data: Vec<f64> = comm.recv().unwrap();
                    comm.send(&s_buf).unwrap();
                }
            }
        }
        count *= 2;
    }
}
