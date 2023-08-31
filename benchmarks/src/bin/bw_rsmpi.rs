//! Rsmpi bandwidth benchmark
use benchmarks::{BandwidthOptions, RsmpiArgs};
use clap::Parser;
use datatypes::{self, DataType};
use mpi::{
    self,
    point_to_point::{Destination, Source},
    traits::{Communicator, Equivalence},
};

fn benchmark<T, P, C>(opts: BandwidthOptions, rank: i32, prepare: P, comm: C) -> Vec<(usize, f32)>
where
    T: Equivalence + Default,
    P: Fn(usize) -> Vec<T>,
    C: Communicator,
{
    let next_rank = (rank + 1) % 2;
    // Preallocated receive buffer
    let mut rbufs: Vec<Vec<T>> = (0..opts.window_size)
        .map(|_| (0..opts.max_size).map(|_| T::default()).collect())
        .collect();
    benchmarks::bw(
        opts,
        rank.try_into().unwrap(),
        prepare,
        |rank, window_size, sbuf| {
            let proc = comm.process_at_rank(next_rank);
            // NOTE: wait_all can only handle requests that all operate on the
            // same type
            mpi::request::multiple_scope(window_size, |scope, coll| {
                if rank == 0 {
                    let ptr = sbuf[..].as_ptr() as *const u8;
                    let buf = unsafe {
                        std::slice::from_raw_parts(ptr, sbuf.len() * std::mem::size_of::<T>())
                    };
                    for _ in 0..window_size {
                        coll.add(proc.immediate_send(scope, buf));
                    }
                } else {
                    let mut tmp = &mut rbufs[..];
                    for _ in 0..window_size {
                        let (a, b) = tmp.split_at_mut(1);
                        tmp = b;
                        let rbuf = &mut a[0][..sbuf.len()];
                        let ptr = rbuf[..].as_ptr() as *mut u8;
                        let buf = unsafe {
                            std::slice::from_raw_parts_mut(ptr, rbuf.len() * std::mem::size_of::<T>())
                        };
                        let rreq = proc.immediate_receive_into(scope, buf);
                        coll.add(rreq);
                    }
                }
                let mut stats = vec![];
                coll.wait_all(&mut stats);
            });
            if rank == 0 {
                let (_, _): (Vec<i32>, _) = proc.receive_vec();
            } else {
                proc.send(&[0i32]);
            }
        },
    )
}

fn main() {
    let args = RsmpiArgs::parse();
    let opts: BandwidthOptions = benchmarks::load_config(&args.config).unwrap();
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

    for (size, bw) in results {
        println!("{} {}", size, bw);
    }
}
