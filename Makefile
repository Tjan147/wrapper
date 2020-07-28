PROJ_HOME := $(PWD)

clean:
	cd rust && cargo clean

build:
	cd rust && cargo build --release
	cd $(PROJ_HOME)
	go build

test: build
	go test -v