use std::time::Instant;
use serde::Deserialize;
use datatypes::DataType;

#[derive(Debug, Deserialize)]
pub struct LatencyOptions {
    pub iterations: usize,
    pub skip: usize,
    pub warmup_validation: usize,
    pub min_size: usize,
    pub max_size: usize,
    pub datatype: DataType,
}

/// Generic latency benchmark function.
///
/// The `prepare` callback is used to prepare data for an iteration. The
/// `body0` and `body1` callbacks are called on rank 0 and 1 of the
/// communicator respectively.
pub fn latency<T, P, B0, B1>(
    opts: LatencyOptions,
    rank: isize,
    prepare: P,
    body0: B0,
    body1: B1,
) -> Vec<(usize, f32)>
where
    P: Fn(usize) -> T,
    B0: Fn(&T),
    B1: Fn(&T),
{
    let mut results = vec![];
    let mut size = opts.min_size;
    while size <= opts.max_size {
        let mut total_time = 0.0;
        let data = prepare(size);
        for i in 0..opts.iterations + opts.skip {
            if rank == 0 {
                for j in 0..=opts.warmup_validation {
                    let start = Instant::now();
                    body0(&data);
                    if i >= opts.skip && j == opts.warmup_validation {
                        total_time += Instant::now()
                            .duration_since(start)
                            .as_secs_f32();
                    }
                }
            } else {
                for j in 0..=opts.warmup_validation {
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
