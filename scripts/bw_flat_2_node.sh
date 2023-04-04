#!/bin/sh
#SBATCH -N 2

source ./scripts/2node.sh
source $SAFE_MPI_ENV_FILE
srun -w $SERVER -N 1 ./target/release/bw_flat -c $SAFE_MPI_CONFIG -p 8888 -s $SERVER_IP &
sleep 2
srun -N 1 ./target/release/bw_flat -c $SAFE_MPI_CONFIG -p 8888 $SERVER_IP
