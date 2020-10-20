#!/usr/bin/env bash

set -x

#
# https://github.com/filecoin-project/filecoin-ffi/blob/master/README.md
# the `go get` part
#
go get github.com/filecoin-project/filecoin-ffi
git submodule add https://github.com/filecoin-project/filecoin-ffi.git extern/ffi
make -C ./extern/ffi
go mod edit -replace=github.com/filecoin-project/filecoin-ffi=./extern/ffi
