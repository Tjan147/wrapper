PROJ_HOME := $(dir $(realpath $(lastword $(MAKEFILE_LIST))))

all: report
.PHONY: all

check-tools:
	@bash $(PROJ_HOME)scripts/check_toolchain.sh
.PHONY: check-tools

install-deps:
	@bash $(PROJ_HOME)scripts/install_ffi.sh $(PROJ_HOME)
.PHONY: install-deps

go-build: check-tools install-deps
	cd $(PROJ_HOME) && go build -o $(PROJ_HOME)bin/bench ./cmd
.PHONY: go-build

fetch-params:
	@bash $(PROJ_HOME)scripts/fetch_params.sh $(PROJ_HOME)
.PHONY: fetch-params

bench: fetch-params
	bash $(PROJ_HOME)scripts/run_bench.sh $(PROJ_HOME) sample 2K
.PHONY: bench
