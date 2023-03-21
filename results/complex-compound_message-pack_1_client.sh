#!/bin/sh
#SBATCH -o results/complex-compound_message-pack_1_client.out
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.2 -p 7776 -k message-pack -c ./inputs/complex-compound.yaml
