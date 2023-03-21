#!/bin/sh
#SBATCH -o results-2023-03-21/simple_iovec_3_server.out
#SBATCH -w er06
source $HOME/ompi-install/env
target/release/latency_iovec 172.16.0.6 -p 1347 -c ./inputs/simple.yaml -s
