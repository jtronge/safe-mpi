use benchmarks::{data_controllers::FlatController, IovecArgs, LatencyOptions};
use clap::Parser;
use datatypes::DataType;
use flat::FlatBuffer;
use std::net::SocketAddr;

fn benchmark<T, P>(args: IovecArgs, opts: LatencyOptions, prepare: P) -> Vec<(usize, f32)>
where
    T: FlatBuffer + Default,
    P: Fn(usize) -> Vec<T>,
{
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server).expect("Failed to initialize safe_mpi");
    let world = FlatController::new(sm.world());

    let rank = if args.server { 0 } else { 1 };
    // Set up the receive buffers
    let mut rbuf0: Vec<T> = (0..opts.max_size).map(|_| T::default()).collect();
    let mut rbuf1: Vec<T> = (0..opts.max_size).map(|_| T::default()).collect();
    benchmarks::latency(
        opts,
        rank.try_into().unwrap(),
        prepare,
        |sbuf| {
            world.send(sbuf, 0).unwrap();
            world.recv(&mut rbuf0[..sbuf.len()], 0).unwrap();
        },
        |sbuf| {
            world.recv(&mut rbuf1[..sbuf.len()], 0).unwrap();
            world.send(sbuf, 0).unwrap();
        },
    )
}

fn main() {
    let args = IovecArgs::parse();
    let opts: LatencyOptions = benchmarks::load_config(&args.config).unwrap();

    let results = match opts.datatype {
        DataType::Simple => benchmark(args, opts, datatypes::simple),
        DataType::ComplexNoncompound => benchmark(args, opts, datatypes::complex_noncompound),
        DataType::ComplexCompound => panic!("complex compound is not supported"),
    };

    for (size, lat) in results {
        println!("{} {}", size, lat);
    }
}
