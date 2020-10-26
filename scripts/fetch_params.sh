#!/usr/bin/env bash

set -eo pipefail

PROJ_HOME="$1"
SECTOR_SIZE="$2"

PARAM_SIZE=
case "$SECTOR_SIZE" in
    "2K")
        PARAM_SIZE=2048
        ;;
    "8M")
        PARAM_SIZE=8388608
        ;;
    "512M")
        PARAM_SIZE=536870912
        ;;
    "32G")
        PARAM_SIZE=34359738368
        ;;
    *)
        echo "Unknown sector size $SECTOR_SIZE, only (2K|8M|512M|32G) sized sector available!"
        exit 1
        ;;
esac


. "$PROJ_HOME"/scripts/set_env.sh "$PROJ_HOME"


if [ ! -d "$PROJ_HOME/extern/paramfetch" ]
then
    cd "$PROJ_HOME" || exit
    git submodule add https://github.com/filecoin-project/go-paramfetch.git extern/paramfetch
fi

if [ ! -f "$PROJ_HOME/extern/paramfetch/go-paramfetch" ]
then
    cd "$PROJ_HOME/extern/paramfetch" || exit
    go mod edit -replace=github.com/filecoin-project/go-paramfetch=./
    go build -o ./go-paramfetch ./paramfetch
    cd ../..
fi

"$PROJ_HOME"/extern/paramfetch/go-paramfetch "$PARAM_SIZE" "$PROJ_HOME/extern/ffi/parameters.json"

echo "cached filecoin parameters ready!"
