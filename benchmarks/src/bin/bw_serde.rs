use std::net::SocketAddr;
use clap::Parser;
use serde::{Serialize, de::DeserializeOwned};
use benchmarks::{
    SerdeArgs,
    BandwidthOptions,
    SerKind,
    data_controllers::{
        SerdeScope,
        SerdeController,
        BincodeController,
        MessagePackController,
        PostcardController,
    },
};
use datatypes::DataType;
use safe_mpi;

fn serde_bw<T, P, S>(
    opts: BandwidthOptions,
    rank: usize,
    comm: S,
    prepare: P,
) -> Vec<(usize, f32)>
where
    T: Serialize + DeserializeOwned,
    P: Fn(usize) -> Vec<T>,
    S: SerdeController,
{

    benchmarks::bw(
        opts,
        rank,
        prepare,
        |rank, window_size, sbuf| {
            comm.scope(|scope| {
                let mut reqs = vec![];
                if rank == 0 {
                    for j in 0..window_size {
                        reqs.push(scope.isend(sbuf, 0).unwrap());
                    }
                } else {
                    for j in 0..window_size {
                        reqs.push(scope.irecv(0).unwrap());
                    }
                }
                scope.wait_all(&reqs[..]);
                // Extract/deserialize any data
                for req in reqs {
                    let _ = scope.data::<T>(req);
                }
            });
            if rank == 0 {
                let _ = comm.recv::<Vec<usize>>(0);
            } else {
                comm.send(&[0], 0);
            }
        },
    )
}

fn benchmark<T, P>(args: SerdeArgs, opts: BandwidthOptions, prepare: P) -> Vec<(usize, f32)>
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
            serde_bw(opts, rank, comm, prepare)
        }
        SerKind::Postcard => {
            let comm = PostcardController::new(world);
            serde_bw(opts, rank, comm, prepare)
        }
        SerKind::Bincode => {
            let comm = BincodeController::new(world);
            serde_bw(opts, rank, comm, prepare)
        }
    }
}

fn main() {
    let args = SerdeArgs::parse();
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
