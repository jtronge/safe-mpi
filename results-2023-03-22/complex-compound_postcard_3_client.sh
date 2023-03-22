#!/bin/sh
#SBATCH -o results-2023-03-22/complex-compound_postcard_3_client.out
#SBATCH -N 1
source /home/jtronge/ompi-install3/env
./target/release/latency_serde 172.16.0.6 -p 7776 -k postcard -c ./inputs/complex-compound.yaml
