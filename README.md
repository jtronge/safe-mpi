# safe-mpi

This is an experimental memory-safe implementation of point-to-point messaging
using UCX.

## Compiling

This project requires Rust (rustc 1.64.0 was used for the inital tests), an MPI
installation, and a UCX installation (at least version 1.12).

Build everything with `cargo build --release`. `--release` is necessary for
running the benchmarks.
