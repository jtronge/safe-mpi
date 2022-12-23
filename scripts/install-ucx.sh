#!/bin/sh
TMP=/tmp

cd $TMP
curl -O -L https://github.com/openucx/ucx/releases/download/v1.13.1/ucx-1.13.1.tar.gz
tar -xvf ucx-1.13.1.tar.gz
cd ucx-1.13.1
./configure --prefix=$PREFIX
make
make install
