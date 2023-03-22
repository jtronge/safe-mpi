#!/bin/sh
#SBATCH -o results-2023-03-22/simple_iovec_3_server.out
#SBATCH -N 1
#SBATCH -w er04
source /home/jtronge/ompi-install3/env
target/release/latency_iovec 172.16.0.4 -p 1347 -c ./inputs/simple.yaml -s
