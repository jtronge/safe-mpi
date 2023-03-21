#!/bin/sh
#SBATCH -o results/complex-noncompound_postcard_2_client.out
source $HOME/ompi-install/env
./target/release/latency_serde 172.16.0.2 -p 7776 -k postcard -c ./inputs/complex-noncompound.yaml
