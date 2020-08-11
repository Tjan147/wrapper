PROJ_HOME := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

gohome:
	cd $(PROJ_HOME)

clean: gohome
	cd rust && cargo clean && rm -f ./wrapper
	rm -rf rust/sample
	rm -rf sample

build_rust: gohome
	cd rust && cargo build --release

build: build_rust gohome
	go build

test: build
	go test -v