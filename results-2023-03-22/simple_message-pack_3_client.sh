#!/bin/sh
#SBATCH -o results-2023-03-22/simple_message-pack_3_client.out
#SBATCH -N 1
source /home/jtronge/ompi-install3/env
./target/release/latency_serde 172.16.0.4 -p 7776 -k message-pack -c ./inputs/simple.yaml
