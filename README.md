# safe-mpi

This is an experimental memory-safe implementation of point-to-point messaging
using UCX. This experiments with a few different types of message formats that
allow for memory safety, including serialization and using rust type IDs with
flat buffers and datatypes broken up into iovecs.

## Compiling

This project requires Rust (rustc 1.64.0 was used for the inital tests), an MPI
installation, and a UCX installation (at least version 1.12).

Build everything with `cargo build --release`. `--release` is necessary for
running the benchmarks.

## Benchmarks

The benchmarks are designed to run with Slurm. They can be run with the
`scripts/benchmark.py` script which takes a couple of options and a config.
There are a couple different configs in `benchmark-configs`.

Output is a JSON file which can be graphed with `scripts/graph.py`.

## Crates and Subdirectories

### datatypes

This contains a set of datatypes and serialization implementations that can be
used for benchmarking.

### benchmarks

The actual benchmarking code, output and comparison code.

### safe-mpi

Code for interacting with UCX and sending/receiving messages.

### scripts

This includes the run, benchmarking, and graphing scripts.
