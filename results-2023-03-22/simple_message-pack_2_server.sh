#!/bin/sh
#SBATCH -o results-2023-03-22/simple_message-pack_2_server.out
#SBATCH -N 1
#SBATCH -w er03
source /home/jtronge/ompi-install2/env
./target/release/latency_serde 172.16.0.3 -p 7776 -k message-pack -c ./inputs/simple.yaml -s
