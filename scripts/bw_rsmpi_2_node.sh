#!/bin/sh
#SBATCH -N 2

source $SAFE_MPI_ENV_FILE
mpirun -np 2 -N 1 ./target/release/bw_rsmpi -c $SAFE_MPI_CONFIG
