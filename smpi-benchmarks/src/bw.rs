//! Bandwidth benchmark code
use serde::Deserialize;
use std::time::Instant;

#[derive(Debug, Deserialize)]
pub struct BandwidthOptions {
    pub min_size: usize,
    pub max_size: usize,
    pub window_size: usize,
    pub iterations: usize,
    pub skip: usize,
    pub warmup_validation: usize,
}

/// Generic bandwidth benchmark function. Returns a vec of pairs of the form
/// (size, MB/s).
///
/// Based on the OSU microbenchmarks version for MPI.
pub fn bw<T, P, B>(
    opts: BandwidthOptions,
    rank: usize,
    prepare: P,
    mut body: B,
) -> Vec<(usize, f32)>
where
    P: Fn(usize) -> T,
    B: FnMut(usize, usize, &T),
{
    let mut results = vec![];
    let mut size = opts.min_size;

    while size <= opts.max_size {
        let mut total_time = 0.0;
        // Prepare the send buffer
        let s_buf = prepare(size);
        for i in 0..(opts.iterations + opts.skip) {
            if rank == 0 {
                for k in 0..=opts.warmup_validation {
                    let start = Instant::now();
                    body(rank, opts.window_size, &s_buf);
                    if i >= opts.skip && k == opts.warmup_validation {
                        // The osu version includes another factor that I'm not
                        // sure is necessary
                        total_time += Instant::now().duration_since(start).as_secs_f32();
                    }
                }
            } else {
                for _ in 0..=opts.warmup_validation {
                    body(rank, opts.window_size, &s_buf);
                }
            }
        }
        if rank == 0 {
            let bandwidth =
                (size as f32 / 1.0e6 * opts.iterations as f32 * opts.window_size as f32)
                    / total_time;
            results.push((size, bandwidth));
        }
        size *= 2;
    }

    results
}
