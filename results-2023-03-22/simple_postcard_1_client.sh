#!/bin/sh
#SBATCH -o results-2023-03-22/simple_postcard_1_client.out
#SBATCH -N 1
source /home/jtronge/ompi-install2/env
./target/release/latency_serde 172.16.0.3 -p 7776 -k postcard -c ./inputs/simple.yaml
