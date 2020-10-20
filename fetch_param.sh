#!/usr/bin/env bash

set -exo pipefail

#
# https://github.com/filecoin-project/rust-fil-proofs\#parameter-file-location
#
export FIL_PROOFS_PARAMETER_CACHE=./extern/params

if [ ! -d ./extern/paramfetch ]
then
    git submodule add https://github.com/filecoin-project/go-paramfetch.git extern/paramfetch
fi

cd ./extern/paramfetch
go mod edit -replace=github.com/filecoin-project/go-paramfetch=./
go build -o ./go-paramfetch ./paramfetch
./go-paramfetch 2048 ../ffi/parameters.json
cd ../..
