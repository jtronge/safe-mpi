#!/bin/sh
#SBATCH -o results/complex-noncompound_iovec_3_client.out
source $HOME/ompi-install/env
target/release/latency_iovec 172.16.0.2 -p 1347 -c ./inputs/complex-noncompound.yaml
