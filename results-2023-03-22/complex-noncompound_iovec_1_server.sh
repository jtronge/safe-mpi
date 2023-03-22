#!/bin/sh
#SBATCH -o results-2023-03-22/complex-noncompound_iovec_1_server.out
#SBATCH -N 1
#SBATCH -w er04
source /home/jtronge/ompi-install3/env
target/release/latency_iovec 172.16.0.4 -p 1347 -c ./inputs/complex-noncompound.yaml -s
