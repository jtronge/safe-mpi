use clap::Parser;
use mpi::{
    self,
    traits::Communicator as Communicator1,
    point_to_point::{Source, Destination, Status},
};
use benchmarks::RsmpiArgs;
use datatypes::{
    self,
    ComplexNoncompound,
};

fn main() {
    let args = RsmpiArgs::parse();
    let opts = benchmarks::load_config(&args.config).unwrap();
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();
    let next_rank = (rank + 1) % size;

    assert_eq!(size, 2);
    let prepare = datatypes::simple;
    let results = benchmarks::latency(
        opts,
        rank.try_into().unwrap(),
        prepare,
        |s_buf| {
            world
                .process_at_rank(next_rank)
                .send(&s_buf[..]);
            let (_data, _status): (Vec<i32>, Status) = world
                .process_at_rank(next_rank)
                .receive_vec();
        },
        |s_buf| {
            let (_data, _status): (Vec<i32>, Status) = world
                .process_at_rank(next_rank)
                .receive_vec();
            world
                .process_at_rank(next_rank)
                .send(&s_buf[..]);
        },
    );
    // TODO
    // vec![]

    for (size, lat) in results {
        println!("{} {}", size, lat);
    }
}
