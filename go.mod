module github.com/tjan147/wrapper

go 1.14

require (
	github.com/filecoin-project/filecoin-ffi v0.30.3
	github.com/filecoin-project/go-state-types v0.0.0-20200928172055-2df22083d8ab
	github.com/filecoin-project/specs-actors v0.9.12
	github.com/ipfs/go-cid v0.0.7
	github.com/stretchr/testify v1.6.1
)

replace github.com/filecoin-project/filecoin-ffi => ./extern/ffi
