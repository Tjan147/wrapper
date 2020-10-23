#!/usr/bin/env bash

PROJ_HOME="$1"

. $PROJ_HOME/scripts/set_env.sh

set -exo pipefail

if [ ! -d $PROJ_HOME/extern/paramfetch ]
then
    git submodule add https://github.com/filecoin-project/go-paramfetch.git extern/paramfetch
fi

if [ ! -f $PROJ_HOME/extern/paramfetch/go-paramfetch ]
then
    cd $PROJ_HOME/extern/paramfetch
    go mod edit -replace=github.com/filecoin-project/go-paramfetch=./
    go build -o ./go-paramfetch ./paramfetch
    cd ../..
fi

if [ ! "$(ls -A "$FIL_PROOFS_PARAMTER_CACHE")" ]
then
    $PROJ_HOME/extern/paramfetch/go-paramfetch 2048 $PROJ_HOME/extern/ffi/parameters.json
fi


