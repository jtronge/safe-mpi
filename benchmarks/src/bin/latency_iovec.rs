use std::net::SocketAddr;
use clap::Parser;
use benchmarks::{
    LatencyOptions,
    IovecArgs,
    data_controllers::IovecController,
};
use datatypes::{self, DataType};
use iovec::ChunkSerDe;

fn benchmark<T, P>(args: IovecArgs, opts: LatencyOptions, prepare: P) -> Vec<(usize, f32)>
where
    T: ChunkSerDe,
    P: Fn(usize) -> T,
{
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to initialize safe_mpi");
    let world = IovecController::new(sm.world());

    let rank = if args.server { 0 } else { 1 };
    benchmarks::latency(
        opts,
        rank,
        prepare,
        |s_buf| {
            world.send(s_buf, 0).unwrap();
            let _data: T = world.recv(0).unwrap();
        },
        |s_buf| {
            let _data: T = world.recv(0).unwrap();
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
