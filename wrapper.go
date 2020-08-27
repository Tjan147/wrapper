package wrapper

/*
#cgo LDFLAGS: -L./rust/target/release/ -lwrapper
#include "rust/wrapper.h"
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"fmt"
)

// CallGenerateSampleFile wraps the
// `char *generate_sample_file(uint32_t expected_size, const char *path_cstr)`
func CallGenerateSampleFile(expectedSize uint32, expectedPath string) error {
	ptr := C.generate_sample_file(C.uint32_t(expectedSize), C.CString(expectedPath))
	defer C.release(ptr)

	res := C.GoString(ptr)
	if len(res) > 0 {
		return fmt.Errorf(res)
	}
	return nil
}

// CallCountNodeNum wraps the `uint32_t count_node_num(const char *path_cstr)`
func CallCountNodeNum(filePath string) uint32 {
	return uint32(C.count_node_num(C.CString(filePath)))
}
