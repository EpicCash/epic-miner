#!/bin/sh

cd /opt/epic-miner-cuda
export LD_LIBRARY_PATH=/opt/epic-miner-cuda/lib
exec ./bin/epic-miner
