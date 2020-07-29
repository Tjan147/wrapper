package wrapper

/*
#cgo LDFLAGS: -L./rust/target/release -lwrapper
#include "./rust/wrapper.h"
*/
import "C"

func callRustSample(name string) {
	C.hello(C.CString(name))
}
