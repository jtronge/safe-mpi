#!/bin/sh
#SBATCH -o results-2023-03-22/simple_iovec_1_client.out
#SBATCH -N 1
source /home/jtronge/ompi-install2/env
target/release/latency_iovec 172.16.0.3 -p 1347 -c ./inputs/simple.yaml
