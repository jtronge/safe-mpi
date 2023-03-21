#!/bin/sh
#SBATCH -o results/complex-compound_bincode_0_client.out
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.2 -p 7776 -k bincode -c ./inputs/complex-compound.yaml
