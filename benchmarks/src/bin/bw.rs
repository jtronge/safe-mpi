use std::net::SocketAddr;
use clap::Parser;
use safe_mpi::{
    self,
};
use serde::{Serialize, Deserialize};
use nalgebra::Matrix3;
use benchmarks::SerdeArgs;

#[derive(Serialize, Deserialize, Debug)]
struct TestData {
    x: [f64; 2],
}

const MIN_MESSAGE_SIZE: usize = 2;
const MAX_MESSAGE_SIZE: usize = 8;
const ITERATIONS: usize = 16;
const SKIP: usize = 128;
const WARMUP_VALIDATION: usize = 16;

fn main() {
    let args = SerdeArgs::parse();
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to initialize safe_mpi");
    let comm = sm.world();
    let mut size = MIN_MESSAGE_SIZE;
    while size <= MAX_MESSAGE_SIZE {
        size *= 2;
        for _ in 0..ITERATIONS {
/*
            if args.server {
                let req = comm.irecv();
                let data: Vec<Matrix3<f64>> = req.finish().unwrap();
                println!("data: {:?}", data);
            } else {
               // TODO: This could of course just be sent by pointer
               let data: Vec<Matrix3<f64>> = (0..size).map(
                   |_| Matrix3::<f64>::new(1.0, 2.0, 3.0,
                                           4.0, 5.0, 6.0,
                                           7.0, 8.0, 9.0)
               ).collect();
               // println!("Sending data");
               let req = comm.isend(data);
               req.finish().unwrap();
            }
*/
        }
    }
}
