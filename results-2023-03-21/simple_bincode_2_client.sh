#!/bin/sh
#SBATCH -o results-2023-03-21/simple_bincode_2_client.out
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.6 -p 7776 -k bincode -c ./inputs/simple.yaml
