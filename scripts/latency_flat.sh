#!/bin/sh
#SBATCH -n 2
#SBATCH -N 1

source $SAFE_MPI_ENV_FILE
./target/release/latency_flat -c $SAFE_MPI_CONFIG -s -p 8888 127.0.0.1 &
sleep 2
./target/release/latency_flat -c $SAFE_MPI_CONFIG -p 8888 127.0.0.1
