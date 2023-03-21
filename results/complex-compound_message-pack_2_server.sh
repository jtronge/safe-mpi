#!/bin/sh
#SBATCH -o results/complex-compound_message-pack_2_server.out
#SBATCH -w er02
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.2 -p 7776 -k message-pack -c ./inputs/complex-compound.yaml -s
