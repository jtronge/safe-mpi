use benchmarks::{data_controllers::IovecController, IovecArgs, LatencyOptions};
use clap::Parser;
use datatypes::DataType;
use iovec::ChunkSerDe;
use std::net::SocketAddr;

fn benchmark<T, P>(args: IovecArgs, opts: LatencyOptions, prepare: P) -> Vec<(usize, f32)>
where
    T: ChunkSerDe,
    P: Fn(usize) -> Vec<T>,
{
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server).expect("Failed to initialize safe_mpi");
    let world = IovecController::new(sm.world());

    let rank = if args.server { 0 } else { 1 };
    benchmarks::latency(
        opts,
        rank,
        prepare,
        |s_buf| {
            let _size = world.send(s_buf, 0).unwrap();
            let _data: Vec<T> = world.recv(0).unwrap();
        },
        |s_buf| {
            let _data: Vec<T> = world.recv(0).unwrap();
            world.send(s_buf, 0).unwrap();
        },
    )
}

fn main() {
    let args = IovecArgs::parse();
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
