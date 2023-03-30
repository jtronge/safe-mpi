#!/bin/sh
#SBATCH -n 2
#SBATCH -N 1

source $SAFE_MPI_ENV_FILE
mpirun -np 2 ./target/release/latency_rsmpi -c $SAFE_MPI_CONFIG
