#!/usr/bin/env bash

set -eo pipefail

PROJ_HOME="$1"
OUT_DIR="$2"
SECTOR_SIZE="$3"

. "$PROJ_HOME"/scripts/set_env.sh
echo "$FIL_PROOFS_PARAMETER_CACHE"

"$PROJ_HOME"/bin/bench "$OUT_DIR" "$SECTOR_SIZE" &

PID="$!"
sleep 5

while ps -p $PID > /dev/null
do
    ps -p $PID -o %cpu,rss | tail -1 >> "$OUT_DIR"/profile.txt
    sleep 5
done 
