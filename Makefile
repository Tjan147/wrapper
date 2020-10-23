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
	cd $(PROJ_HOME) && go build -o bin/bench ./cmd
.PHONY: go-build

report:
	bash $(PROJ_HOME)scripts/run_bench.sh sample 2K
.PHONY: report
