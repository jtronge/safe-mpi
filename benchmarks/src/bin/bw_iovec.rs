use benchmarks::{
    data_controllers::{wait_all, IovecController},
    BandwidthOptions, IovecArgs,
};
use clap::Parser;
use datatypes::DataType;
use iovec::ChunkSerDe;
use std::net::SocketAddr;

fn benchmark<T, P>(args: IovecArgs, opts: BandwidthOptions, prepare: P) -> Vec<(usize, f32)>
where
    T: ChunkSerDe,
    P: Fn(usize) -> Vec<T>,
{
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server).expect("Failed to initialize safe_mpi");
    let world = IovecController::new(sm.world());

    let rank = if args.server { 0 } else { 1 };
    let ack_msg = vec![0i32];
    benchmarks::bw(opts, rank, prepare, |rank, window_size, sbuf| {
        world.scope(|scope| {
            let mut reqs = vec![];
            if rank == 0 {
                for _ in 0..window_size {
                    reqs.push(scope.isend(sbuf, 0).unwrap());
                }
            } else {
                for _ in 0..window_size {
                    reqs.push(scope.irecv(0).unwrap());
                }
            }
            wait_all(scope, &reqs[..]).unwrap();
            for req in reqs {
                let _ = scope.data::<T>(req);
            }
        });
        if rank == 0 {
            let _ = world.recv::<i32>(0).unwrap();
        } else {
            world.send(&ack_msg, 0).unwrap();
        }
    })
}

fn main() {
    let args = IovecArgs::parse();
    let opts: BandwidthOptions = benchmarks::load_config(&args.config).unwrap();

    let results = match opts.datatype {
        DataType::Simple => benchmark(args, opts, datatypes::simple),
        DataType::ComplexNoncompound => benchmark(args, opts, datatypes::complex_noncompound),
        DataType::ComplexCompound => benchmark(args, opts, datatypes::complex_compound),
    };

    for (size, bw) in results {
        println!("{} {}", size, bw);
    }
}
