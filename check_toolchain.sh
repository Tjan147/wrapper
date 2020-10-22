#!/usr/bin/env bash

if ! command -v cargo &> /dev/null
then
    echo "Rust toolchain unavailable, please install the toolchain first!"
    echo "You can read the page: https://www.rust-lang.org/install.html for more information"
    exit 1
fi

if ! command -v go &> /dev/null
then
    echo "Go toolchain unavailable, please install the toolchain first!"
    echo "You can read the page: https://golang.org/doc/install for more information"
fi
