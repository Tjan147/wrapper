package main

import (
	"fmt"
	"path"

	"github.com/tjan147/wrapper"
)

const (
	exampleAttr = 0755
	exampleSize = 1024
)

var (
	exampleDir  = "../sample"
	exampleName = "sample.dat"
)

func main() {
	exampleFile := path.Join(exampleDir, exampleName)

	if err := wrapper.CallInitTargetDir(exampleDir, true); err != nil {
		panic(err)
	}

	if err := wrapper.CallGenSampleFile(exampleSize, exampleFile); err != nil {
		panic(err)
	}

	nodes := wrapper.CallCountNodeNum(exampleFile)
	if nodes == 0 {
		panic("empty file")
	}

	replicaID, err := wrapper.CallGenReplicaID()
	if err != nil {
		panic(err)
	}
	fmt.Printf("Replica ID: %s\n", replicaID)

	sp, err := wrapper.CallGenSetupParams(nodes)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Setup Params: %s\n", sp)

	scfg, err := wrapper.CallGenStoreCfg(nodes, exampleDir)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Store Config: %s\n", scfg)

	if err := wrapper.CallPorepSetup(exampleFile, sp, scfg, replicaID); err != nil {
		panic(err)
	}

	fmt.Printf("\n--------------------------------------\n")
	fmt.Printf("|===> WORKFLOW DONE SUCCESSFULLY <===|\n\n")
	fmt.Printf("--------------------------------------\n")
}
