#!/bin/sh
#SBATCH -o results-2023-03-22/simple_rsmpi_3.out
#SBATCH -N 2
source /home/jtronge/ompi-install3/env
mpirun -np 2 -N 1 ./target/release/latency_rsmpi -c ./inputs/simple.yaml
