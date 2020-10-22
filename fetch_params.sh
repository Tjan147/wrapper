#!/usr/bin/env bash

. ./set_env.sh

set -exo pipefail

if [ ! -d ./extern/paramfetch ]
then
    git submodule add https://github.com/filecoin-project/go-paramfetch.git extern/paramfetch
fi

if [ ! -f ./extern/paramfetch/go-paramfetch ]
then
    cd ./extern/paramfetch
    go mod edit -replace=github.com/filecoin-project/go-paramfetch=./
    go build -o ./go-paramfetch ./paramfetch
    cd ../..
fi

if [ ! "$(ls -A "$FIL_PROOFS_PARAMTER_CACHE")" ]
then
    ./extern/paramfetch/go-paramfetch 2048 ./extern/ffi/parameters.json
fi
