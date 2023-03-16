use clap::Parser;
use mpi::{
    self,
    traits::{Communicator, Equivalence},
    point_to_point::{Source, Destination, Status},
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
    T: Equivalence,
    P: Fn(usize) -> Vec<T>,
    C: Communicator,
{
    let next_rank = (rank + 1) % 2;
    benchmarks::latency(
        opts,
        rank.try_into().unwrap(),
        prepare,
        |s_buf| {
            comm
                .process_at_rank(next_rank)
                .send(&s_buf[..]);
            let (_data, _status): (Vec<T>, Status) = comm
                .process_at_rank(next_rank)
                .receive_vec();
        },
        |s_buf| {
            let (_data, _status): (Vec<T>, Status) = comm
                .process_at_rank(next_rank)
                .receive_vec();
            comm
                .process_at_rank(next_rank)
                .send(&s_buf[..]);
        },
    )
    // TODO
    // vec![]
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
