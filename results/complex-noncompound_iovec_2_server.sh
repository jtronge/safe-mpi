#!/bin/sh
#SBATCH -o results/complex-noncompound_iovec_2_server.out
#SBATCH -w er02
source $HOME/ompi-install/env
target/release/latency_iovec 172.16.0.2 -p 1347 -c ./inputs/complex-noncompound.yaml -s
