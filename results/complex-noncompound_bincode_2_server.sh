#!/bin/sh
#SBATCH -o results/complex-noncompound_bincode_2_server.out
#SBATCH -w er02
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.2 -p 7776 -k bincode -c ./inputs/complex-noncompound.yaml -s
