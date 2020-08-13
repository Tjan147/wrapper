package wrapper

/*
#cgo LDFLAGS: -L./rust/target/release/ -lwrapper
#include "rust/wrapper.h"
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

// CallSentinel works as a cross-compiling sentinel test
func CallSentinel(name string) {
	C.sentinel(C.CString(name))
}

// CallSetup actually call the setup wrapper in rust
func CallSetup(dataPath string, cacheDir string) uint32 {
	return uint32(C.setup(C.CString(dataPath), C.CString(cacheDir)))
}
