# Safe MPI

## Logging

When running the debug version of the code, set `RUST_LOG=${level}` in the
environment before launching the code. This uses the
[env\_logger](https://docs.rs/env_logger/0.10.0/env_logger/) crate to handle
log levels and output.

