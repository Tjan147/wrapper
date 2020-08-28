package wrapper

/*
#cgo LDFLAGS: -L./rust/target/release/ -lwrapper
#include "rust/wrapper.h"
#include <stdlib.h>
#include <stdint.h>
*/
import "C"

import (
	"encoding/json"
	"fmt"
)

func handleUnitResult(res string, prefix string) error {
	if len(res) > 0 {
		return fmt.Errorf("%s: %s", prefix, res)
	}
	return nil
}

func handleJSONResult(res string, prefix string) (string, error) {
	if json.Valid([]byte(res)) {
		return res, nil
	}
	return "", fmt.Errorf("%s: %s", prefix, res)
}


func handlePathResult(res string, prefix string) (string, error) {
	if FileExists(res) {
		return res, nil
	}
	return "", fmt.Errorf("%s: %s", prefix, res)
}

// CallInitTargetDir wraps the routine:
// `char *initialize_target_dir(const char *dir_cstr, bool need_clean)`
func CallInitTargetDir(dirPath string, cleanFirst bool) error {
	ptr := C.initialize_target_dir(C.CString(dirPath), C.bool(cleanFirst))
	defer C.release(ptr)

	return handleUnitResult(C.GoString(ptr), "InitTargetDir")
}

// CallGenSampleFile wraps the routine:
// `char *generate_sample_file(uint32_t expected_size, const char *path_cstr)`
func CallGenSampleFile(expectedSize uint32, expectedPath string) error {
	ptr := C.generate_sample_file(C.uint32_t(expectedSize), C.CString(expectedPath))
	defer C.release(ptr)

	return handleUnitResult(C.GoString(ptr), "GenSampleFile")
}

// CallGenReplicaID wraps the routine: `char *generate_replica_id(void)'
func CallGenReplicaID() (string, error) {
	ptr := C.generate_replica_id()
	defer C.release(ptr)

	return handleJSONResult(C.GoString(ptr), "GenReplicaID")
}

// CallCountNodeNum wraps the routine: `uint32_t count_node_num(const char *path_cstr)`
func CallCountNodeNum(filePath string) uint32 {
	return uint32(C.count_node_num(C.CString(filePath)))
}

// CallGenSetupParams wraps the routine:
// `char *generate_setup_params(uint32_t node_num)`
func CallGenSetupParams(nodes uint32) (string, error) {
	ptr := C.generate_setup_params(C.uint32_t(nodes))
	defer C.release(ptr)

	return handleJSONResult(C.GoString(ptr), "GenSetupParams")
}

// CallGenStoreCfg wraps the routine:
// `char *generate_store_config(uint32_t node_num, const char *dir_cstr)`
func CallGenStoreCfg(nodes uint32, dirPath string) (string, error) {
	ptr := C.generate_store_config(C.uint32_t(nodes), C.CString(dirPath))
	defer C.release(ptr)

	return handleJSONResult(C.GoString(ptr), "GenStoreCfg")
}

// CallPorepSetup wraps the routine:
// ```c
// char *porep_setup(const char *src_path_cstr,
//                   const char *sp_data_cstr,
//                   const char *scfg_data_cstr,
//                   const char *replica_id_cstr)
// ```
func CallPorepSetup(srcPath, sp, scfg, replicaID string) (string, error) {
	ptr := C.porep_setup(
		C.CString(srcPath), C.CString(sp), C.CString(scfg), C.CString(replicaID),
	)
	defer C.release(ptr)

	return handlePathResult(C.GoString(ptr), "PorepSetup")
}

// CallPorepChallenge wraps the routine: `char *generate_challenge(void)`
func CallPorepChallenge() (string, error) {
	ptr := C.generate_challenge()
	defer C.release(ptr)

	return handleJSONResult(C.GoString(ptr), "PorepChallenge")
}

// CallPorepProve wraps the routine:
// ```c
// char *porep_prove(const char *replica_path_cstr,
//                   const char *sp_data_cstr,
//                   const char *replica_id_cstr,
//                   const char *chal_cstr,
//                   const char *proof_path_cstr);
// ```
func CallPorepProve(replicaPath, sp, replicaID, chal, proofPath string) error {
	ptr := C.porep_prove(
		C.CString(replicaPath), C.CString(sp), C.CString(replicaID), C.CString(chal), C.CString(proofPath),
	)
	defer C.release(ptr)

	return handleUnitResult(C.GoString(ptr), "PorepProve")
}

// CallPorepVerify wraps the routine:
// ```c
// char *porep_verify(const char *replica_path_cstr,
//                    const char *sp_data_cstr,
//                    const char *replica_id_cstr,
//                    const char *chal_cstr,
//                    const char *proof_path_cstr);
// ```
func CallPorepVerify(replicaPath, sp, replicaID, chal, proofPath string) error {
	ptr := C.porep_verify(
		C.CString(replicaPath), C.CString(sp), C.CString(replicaID), C.CString(chal), C.CString(proofPath),
	)
	defer C.release(ptr)

	return handleUnitResult(C.GoString(ptr), "PorepVerify")
}