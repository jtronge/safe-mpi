#!/bin/sh
#SBATCH -o results-2023-03-22/complex-noncompound_message-pack_2_server.out
#SBATCH -N 1
#SBATCH -w er04
source /home/jtronge/ompi-install3/env
./target/release/latency_serde 172.16.0.4 -p 7776 -k message-pack -c ./inputs/complex-noncompound.yaml -s
