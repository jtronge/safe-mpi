#!/bin/sh
TMP=/tmp

cd $TMP
curl -O -L https://download.open-mpi.org/release/hwloc/v2.8/hwloc-2.8.0.tar.bz2
tar -xvf hwloc-2.8.0.tar.bz2
cd hwloc-2.8.0
./configure --prefix=$PREFIX
make
make install

cd $TMP
curl -O -L https://github.com/openpmix/openpmix/releases/download/v4.2.2/pmix-4.2.2.tar.bz2
tar -xvf pmix-4.2.2.tar.bz2
cd pmix-4.2.2
./configure --prefix=$PREFIX --with-hwloc=$PREFIX
make
make install
