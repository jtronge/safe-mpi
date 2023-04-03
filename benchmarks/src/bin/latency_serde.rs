use std::net::SocketAddr;
use clap::Parser;
use serde::{Serialize, de::DeserializeOwned};
use safe_mpi;
use benchmarks::{
    LatencyOptions,
    SerdeArgs,
    SerKind,
    data_controllers::{
        SerdeController,
        BincodeController,
        MessagePackController,
        PostcardController,
    },
};
use datatypes::DataType;

fn serde_latency<T, P, S>(
    opts: LatencyOptions,
    rank: usize,
    comm: S,
    prepare: P,
) -> Vec<(usize, f32)>
where
    T: Serialize + DeserializeOwned,
    P: Fn(usize) -> Vec<T>,
    S: SerdeController,
{
    benchmarks::latency(
        opts,
        rank,
        prepare,
        |s_buf| {
            comm.send(s_buf, 0).unwrap();
            let _data: Vec<T> = comm.recv(0).unwrap();
        },
        |s_buf| {
            let _data: Vec<T> = comm.recv(0).unwrap();
            comm.send(s_buf, 0).unwrap();
        },
    )
}

fn benchmark<T, P>(args: SerdeArgs, opts: LatencyOptions, prepare: P) -> Vec<(usize, f32)>
where
    T: Serialize + DeserializeOwned,
    P: Fn(usize) -> Vec<T>,
{
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to initialize safe_mpi");
    let world = sm.world();

    let rank = if args.server { 0 } else { 1 };
    match args.kind {
        SerKind::MessagePack => {
            let comm = MessagePackController::new(world);
            serde_latency(opts, rank, comm, prepare)
        }
        SerKind::Postcard => {
            let comm = PostcardController::new(world);
            serde_latency(opts, rank, comm, prepare)
        }
        SerKind::Bincode => {
            let comm = BincodeController::new(world);
            serde_latency(opts, rank, comm, prepare)
        }
    }
}

fn main() {
    let args = SerdeArgs::parse();
    let opts: LatencyOptions = benchmarks::load_config(&args.config).unwrap();

    let results = match opts.datatype {
        DataType::Simple => benchmark(args, opts, datatypes::simple),
        DataType::ComplexNoncompound => benchmark(args, opts, datatypes::complex_noncompound),
        DataType::ComplexCompound => benchmark(args, opts, datatypes::complex_compound),
    };

    for (size, lat) in results {
        println!("{} {}", size, lat);
    }
}
