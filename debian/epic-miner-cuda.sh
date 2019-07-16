#!/bin/sh

cd /opt/epic-miner-cuda
export LD_LIBRARY_PATH=/opt/epic-miner-cuda/lib:$LD_LIBRARY_PATH
exec ./bin/epic-miner
