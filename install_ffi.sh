#!/usr/bin/env bash

set -x

# check if the ffi is already built 
#
if [ -f "./extern/ffi/filecoin.h" ] && [ -f "./extern/ffi//libfilcrypto.a" ] && [ -f "./extern/ffi//filcrypto.pc" ]; then
    exit
fi

# read the page: https://github.com/filecoin-project/filecoin-ffi#go-get
#
go get github.com/filecoin-project/filecoin-ffi
git submodule add https://github.com/filecoin-project/filecoin-ffi.git extern/ffi
make -C ./extern/ffi
go mod edit -replace=github.com/filecoin-project/filecoin-ffi=./extern/ffi
