use std::net::SocketAddr;
use clap::Parser;
use benchmarks::{
    IovecArgs,
    BandwidthOptions,
    data_controllers::{
        FlatController,
        wait_all,
    },
};
use datatypes::DataType;
use flat::FlatBuffer;

fn benchmark<T, P>(args: IovecArgs, opts: BandwidthOptions, prepare: P) -> Vec<(usize, f32)>
where
    T: FlatBuffer + Default,
    P: Fn(usize) -> Vec<T>,
{
    let sockaddr = SocketAddr::from((args.address.octets(), args.port));
    let sm = safe_mpi::init(sockaddr, args.server)
        .expect("Failed to initialize safe_mpi");
    let world = FlatController::new(sm.world());

    let rank = if args.server { 0 } else { 1 };
    let mut ack_msg = vec![0i32];
    let mut rbufs: Vec<Vec<T>> = (0..opts.window_size)
        .map(|_| (0..opts.max_size).map(|_| T::default()).collect())
        .collect();
    benchmarks::bw(
        opts,
        rank,
        prepare,
        |rank, window_size, sbuf| {
            world.scope(|scope| {
                let mut reqs = vec![];
                if rank == 0 {
                    for j in 0..window_size {
                        reqs.push(scope.isend(sbuf, 0).unwrap());
                    }
                } else {
                    let mut tmp = &mut rbufs[..];
                    for j in 0..window_size {
                        let (a, b) = tmp.split_at_mut(1);
                        tmp = b;
                        let req = scope
                            .irecv(&mut a[0][..sbuf.len()], 0)
                            .unwrap();
                        reqs.push(req);
                    }
                }
                wait_all(scope, &reqs[..]);
            });
            if rank == 0 {
                world.recv(&mut ack_msg[..], 0).unwrap();
            } else {
                world.send(&ack_msg[..], 0).unwrap();
            }
        },
    )
}

fn main() {
    let args = IovecArgs::parse();
    let opts: BandwidthOptions = benchmarks::load_config(&args.config).unwrap();

    let results = match opts.datatype {
        DataType::Simple => benchmark(args, opts, datatypes::simple),
        DataType::ComplexNoncompound => benchmark(args, opts, datatypes::complex_noncompound),
        DataType::ComplexCompound => panic!("complex compound is not supported"),
    };

    for (size, bw) in results {
        println!("{} {}", size, bw);
    }
}
