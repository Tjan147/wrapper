package wrapper

/*
#cgo LDFLAGS: -L./rust/target/release/ -lwrapper
#include "./rust/wrapper.h"
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

func callRustSample(name string) {
	C.sentinel(C.CString(name))
}

func callSetup(dataPath string, cacheDir string) uint32 {
	return C.setup(C.CString(dataPath), C.CString(cacheDir))
}
