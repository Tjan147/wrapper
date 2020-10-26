#!/usr/bin/env bash

# for more infomation on environment configuration
# please read: https://github.com/filecoin-project/rust-fil-proofs/blob/master/README.md

PROJ_HOME="$1"

# IPFS gateway China cache
#
export IPFS_GATEWAY=https://proof-parameters.s3.cn-south-1.jdcloud-oss.com/ipfs/

# the location to store downloaded parameters
#
export FIL_PROOFS_PARAMETER_CACHE="$PROJ_HOME/extern/params"

# to toggle the downloaded parameter verification
#
#export FIL_PROOFS_VERIFY_PRODUCTION_PARAMS=1

# the location to store the parent's cache data of SDR data structure
#
#export FIL_PROOFS_PARENT_CACHE=/path/to/parent/cache

# the size of parent's cache data of SDR data structure
# WARNING: increase the parent size would increase the requirement of
# host's memory size while make replication computation faster
#
export FIL_PROOFS_SDR_PARENTS_CACHE_SIZE=2048

# to toggle the on-disk parent cache
#
#export FIL_PROOFS_VERIFY_CACHE=1

# to enable multiple cores optimization in PoRep Setup process. 
# WARNING: this feature is based on CPU's shared cache technology
# please refer to the page: https://github.com/filecoin-project/rust-fil-proofs#speed
#
#export FIL_PROOFS_USE_MULTICORE_SDR=1

# to toggle GPU acceleration
# please read the page: https://github.com/filecoin-project/rust-fil-proofs#gpu-usage for more detail
# the supported GPU list can be found in: https://github.com/filecoin-project/bellman#supported--tested-cards
#
#export FIL_PROOFS_USE_GPU_COLUMN_BUILDER=1
#export FIL_PROOFS_USE_GPU_TREE_BUILDER=1
