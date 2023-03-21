#!/bin/sh
#SBATCH -o results/simple_message-pack_2_client.out
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.2 -p 7776 -k message-pack -c ./inputs/simple.yaml
