use std::net::SocketAddr;
use std::time::Instant;
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
        let s_buf: Vec<f64> = (0..count).map(|i| i as f64).collect();
        let mut total_time = 0.0;
        for i in 0..ITERATIONS + SKIP {
            if args.server {
                for j in 0..=WARMUP_VALIDATION {
                    let start = Instant::now();
                    comm.send(&s_buf).unwrap();
                    let _data: Vec<f64> = comm.recv().unwrap();
                    if i >= SKIP && j == WARMUP_VALIDATION {
                        total_time += Instant::now().duration_since(start).as_secs_f32();
                    }
                }
            } else {
                for j in 0..=WARMUP_VALIDATION {
                    let _data: Vec<f64> = comm.recv().unwrap();
                    comm.send(&s_buf).unwrap();
                }
            }
        }
        if args.server {
            let latency = (total_time * 1.0e6) / (2.0 * ITERATIONS as f32);
            println!("{} {}", count, latency);
        }
        count *= 2;
    }
}
