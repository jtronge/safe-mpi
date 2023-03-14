use std::net::SocketAddr;
use std::time::Instant;
use clap::Parser;
use safe_mpi;
use benchmarks::{
    latency,
    LatencyOptions,
    Args,
    Kind,
    data_controllers::{
        SerdeController,
        BincodeController,
        MessagePackController,
        PostcardController,
    },
};
// use nalgebra::Matrix3;

const ITERATIONS: usize = 512;
const SKIP: usize = 16;
const WARMUP_VALIDATION: usize = 8;

fn serde_latency<S>(opts: LatencyOptions, comm: S) -> Vec<(usize, f32)>
where
    S: SerdeController,
{
    benchmarks::latency(
        opts,
        |size: usize| (0..size).map(|i| i as f64).collect::<Vec<f64>>(),
        |s_buf| {
            comm.send(s_buf, 0).unwrap();
            let _data: Vec<f64> = comm.recv(0).unwrap();
        },
        |s_buf| {
            let _data: Vec<f64> = comm.recv(0).unwrap();
            comm.send(s_buf, 0).unwrap();
        }
    )
}

fn main() {
    let args = Args::parse();
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to initialize safe_mpi");
    let world = sm.world();

    let opts = LatencyOptions {
        iterations: ITERATIONS,
        skip: SKIP,
        warmup_validation: WARMUP_VALIDATION,
        min_size: 2,
        max_size: 512,
        rank: if args.server { 0 } else { 1 },
    };
    let results = match args.kind {
        Kind::MessagePack => {
            let comm = MessagePackController::new(world);
            serde_latency(opts, comm)
        }
        Kind::Postcard => {
            let comm = PostcardController::new(world);
            serde_latency(opts, comm)
        }
        Kind::Bincode => {
            let comm = BincodeController::new(world);
            serde_latency(opts, comm)
        }
    };

    for (size, lat) in results {
        println!("{} {}", size, lat);
    }
}
