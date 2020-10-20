#!/usr/bin/env bash

set -exo pipefail

#
# https://github.com/filecoin-project/rust-fil-proofs\#parameter-file-location
#
export IPFS_GATEWAY=https://proof-parameters.s3.cn-south-1.jdcloud-oss.com/ipfs/
export FIL_PROOFS_PARAMETER_CACHE=./extern/params

if [ ! -d ./extern/paramfetch ]
then
    git submodule add https://github.com/filecoin-project/go-paramfetch.git extern/paramfetch
fi

cd ./extern/paramfetch
go mod edit -replace=github.com/filecoin-project/go-paramfetch=./
go build -o ./go-paramfetch ./paramfetch
cd ../..
./extern/paramfetch/go-paramfetch 2048 ./extern/ffi/parameters.json
