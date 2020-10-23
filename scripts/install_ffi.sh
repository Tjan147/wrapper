#!/usr/bin/env bash

PROJ_HOME="$1"

# based on the page: https://github.com/filecoin-project/filecoin-ffi#go-get

# check if the local submodule is added
#
if [ ! -d "$PROJ_HOME/extern/ffi" ]; then
    cd "$PROJ_HOME" || exit
    go get github.com/filecoin-project/filecoin-ffi # this command will actually return an error
    git submodule add https://github.com/filecoin-project/filecoin-ffi.git extern/ffi
fi

# check if the ffi is already built 
#
if [ ! -f "$PROJ_HOME/extern/ffi/filcrypto.h" ] || [ ! -f "$PROJ_HOME/extern/ffi/libfilcrypto.a" ] || [ ! -f "$PROJ_HOME/extern/ffi/filcrypto.pc" ] 
then
    cd "$PROJ_HOME" || exit
    make -C ./extern/ffi
    go mod edit -replace=github.com/filecoin-project/filecoin-ffi=./extern/ffi
fi

echo "filecoin ffi ready!"
