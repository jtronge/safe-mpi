#!/bin/sh
#SBATCH -o results-2023-03-21/simple_message-pack_0_client.out
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.6 -p 7776 -k message-pack -c ./inputs/simple.yaml
