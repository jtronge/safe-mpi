#!/bin/sh
#SBATCH -o results-2023-03-21/complex-compound_message-pack_2_server.out
#SBATCH -w er06
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.6 -p 7776 -k message-pack -c ./inputs/complex-compound.yaml -s
