PROJ_HOME := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

gohome:
	cd $(PROJ_HOME)

clean: gohome clean_rust clean_c clean_go
.PHONY: clean

build_rust: gohome
	cd rust && cargo build --release

clean_rust: gohome
	cd rust && cargo clean && rm wrapper.h

build_c: gohome build_rust
	gcc --std=c11 -o bin/runner runner.c -Lrust/target/release -lwrapper

clean_c: gohome
	rm -rf bin

build_go: build_rust gohome
	go build

clean_go: gohome
	rm -rf sample

test: build
	go test -v