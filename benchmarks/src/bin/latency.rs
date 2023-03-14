use std::net::SocketAddr;
use std::time::Instant;
use clap::Parser;
use safe_mpi::{
    self,
    data_controllers::{
        BincodeController,
        MessagePackController,
        PostcardController,
    },
};
use benchmarks::{
    latency,
    LatencyOptions,
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
    // let comm = MessagePackController::new(sm.world());
    let comm = BincodeController::new(sm.world());
    // let comm = PostcardController::new(sm.world());

    let results = benchmarks::latency(
        LatencyOptions {
            iterations: ITERATIONS,
            skip: SKIP,
            warmup_validation: WARMUP_VALIDATION,
            min_size: 2,
            max_size: 512,
            rank: if args.server { 0 } else { 1 },
        },
        |size: usize| (0..size).map(|i| i as f64).collect::<Vec<f64>>(),
        |s_buf| {
            comm.send(s_buf, 0).unwrap();
            let _data: Vec<f64> = comm.recv(0).unwrap();
        },
        |s_buf| {
            let _data: Vec<f64> = comm.recv(0).unwrap();
            comm.send(s_buf, 0).unwrap();
        }
    );

    for (size, lat) in results {
        println!("{} {}", size, lat);
    }
}
