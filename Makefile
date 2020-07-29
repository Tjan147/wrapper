PROJ_HOME := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

gohome:
	cd $(PROJ_HOME)

clean: gohome
	cd rust && cargo clean && rm -f ./wrapper

build_rust: gohome
	cd rust && cargo build --release
	cd rust && cbindgen --config ./cbindgen.toml --crate wrapper --output ./wrapper.h

build: build_rust gohome
	go build

test: build
	go test -v