//! Latency benchmark code
use serde::Deserialize;
use std::time::Instant;

#[derive(Debug, Deserialize)]
pub struct LatencyOptions {
    pub iterations: usize,
    pub skip: usize,
    pub warmup_validation: usize,
    pub min_size: usize,
    pub max_size: usize,
}

/// Generic latency benchmark function. Returns a vec of pairs of the form
/// (size, microseconds).
///
/// The `prepare` callback is used to prepare data for an iteration. The
/// `body0` and `body1` callbacks are called on rank 0 and 1 of the
/// communicator respectively.
///
/// Based on the OSU microbenchmarks version for MPI.
pub fn latency<T, P, B0, B1>(
    opts: LatencyOptions,
    rank: usize,
    prepare: P,
    mut body0: B0,
    mut body1: B1,
) -> Vec<(usize, f32)>
where
    P: Fn(usize) -> T,
    B0: FnMut(&T),
    B1: FnMut(&T),
{
    let mut results = vec![];
    let mut size = opts.min_size;
    while size <= opts.max_size {
        let mut total_time = 0.0;
        // Prepare the send buffer
        let data = prepare(size);
        for i in 0..opts.iterations + opts.skip {
            if rank == 0 {
                for j in 0..=opts.warmup_validation {
                    let start = Instant::now();
                    body0(&data);
                    if i >= opts.skip && j == opts.warmup_validation {
                        total_time += Instant::now().duration_since(start).as_secs_f32();
                    }
                }
            } else {
                for _ in 0..=opts.warmup_validation {
                    body1(&data);
                }
            }
        }
        if rank == 0 {
            let latency = (total_time * 1.0e6) / (2.0 * opts.iterations as f32);
            results.push((size, latency));
        }
        size *= 2;
    }
    results
}
