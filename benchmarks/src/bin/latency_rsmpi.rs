use clap::Parser;
use mpi::{
    self,
    traits::{Communicator, Equivalence},
    point_to_point::{Source, Destination},
};
use benchmarks::{
    RsmpiArgs,
    LatencyOptions,
};
use datatypes::{
    self,
    DataType,
};

fn benchmark<T, P, C>(opts: LatencyOptions, rank: i32, prepare: P, comm: C) -> Vec<(usize, f32)>
where
    T: Equivalence + Default,
    P: Fn(usize) -> Vec<T>,
    C: Communicator,
{
    let next_rank = (rank + 1) % 2;
    // Set up the receive buffers
    let mut rbuf0: Vec<T> = (0..opts.max_size)
        .map(|_| T::default())
        .collect();
    let mut rbuf1: Vec<T> = (0..opts.max_size)
        .map(|_| T::default())
        .collect();
    benchmarks::latency(
        opts,
        rank.try_into().unwrap(),
        prepare,
        |s_buf| {
            comm
                .process_at_rank(next_rank)
                .send(&s_buf[..]);
            let _ = comm
                .process_at_rank(next_rank)
                .receive_into(&mut rbuf0);
        },
        |s_buf| {
            let _ = comm
                .process_at_rank(next_rank)
                .receive_into(&mut rbuf1);
            comm
                .process_at_rank(next_rank)
                .send(&s_buf[..]);
        },
    )
}

fn main() {
    let args = RsmpiArgs::parse();
    let opts: LatencyOptions = benchmarks::load_config(&args.config).unwrap();
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();
    assert_eq!(size, 2);

    let results = match opts.datatype {
        DataType::Simple => {
            let prepare = datatypes::simple;
            benchmark(opts, rank, prepare, world)
        }
        DataType::ComplexNoncompound => {
            let prepare = datatypes::complex_noncompound;
            benchmark(opts, rank, prepare, world)
        }
        DataType::ComplexCompound => {
            panic!("ComplexCompound is not supported by rsmpi");
        }
    };

    for (size, lat) in results {
        println!("{} {}", size, lat);
    }
}
