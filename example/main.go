package main

import (
	"fmt"
	"flag"
	"path"
	"time"

	"github.com/tjan147/wrapper"
)

const (
	defaultPerm = 0755
	defaultSize = 0
)

// input parameters
var (
	initReplicaDir = flag.Bool("r", false, "flag to toggle the replica directoy initialization")
	genInputFile = flag.Int("g", defaultSize, "flag to toggle a given size file generation(in KB)")
	chalSessionNum = flag.Int("c", 1, "times of challenge-prove-verify session")
	inputFilePath = flag.String("i", "sample.dat", "the input file")
	outputDirPath = flag.String("o", ".", "the replica data cache directory")
)

func session(idx int, replicaOutput, sp, replicaID string) {
	fmt.Printf("===> #%d round challenge-prove-verify session\n", idx)

	proofPath := path.Join(*outputDirPath, fmt.Sprintf("proof_%dth_round.json", idx))

	// challenge 
	chal, err := wrapper.CallPorepChallenge()	
	if err != nil {
		panic(err)
	}
	fmt.Printf("Challenge: %s\n", chal)

	// prove
	start := time.Now()
	if err := wrapper.CallPorepProve(replicaOutput, sp, replicaID, chal, proofPath); err != nil {
		panic(err)
	}
	fmt.Printf("PoRep prove costs %s ...\n", time.Now().Sub(start).String())
	
	// verify
	start = time.Now()
	if err := wrapper.CallPorepVerify(replicaOutput, sp, replicaID, chal, proofPath); err != nil {
		panic(err)
	}
	fmt.Printf("PoRep verify costs %s ...\n", time.Now().Sub(start).String())

	fmt.Printf("===> #%d round challenge-prove-verify session\n", idx)
}

func main() {
	flag.Parse()

	if err := wrapper.CallInitTargetDir(*outputDirPath, *initReplicaDir); err != nil {
		panic(err)
	}

	if *genInputFile > defaultSize {
		if err := wrapper.CallGenSampleFile(uint32(*genInputFile * 1024), *inputFilePath); err != nil {
			panic(err)
		}
	}

	nodes := wrapper.CallCountNodeNum(*inputFilePath)
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

	scfg, err := wrapper.CallGenStoreCfg(nodes, *outputDirPath)
	if err != nil {
		panic(err)
	}
	fmt.Printf("Store Config: %s\n", scfg)

	start := time.Now()
	replicaOutput, err := wrapper.CallPorepSetup(*inputFilePath, sp, scfg, replicaID)
	if err != nil {
		panic(err)
	}
	fmt.Printf("PoRep setup costs %s ...\n", time.Now().Sub(start).String())

	
	for i := 1; i <= *chalSessionNum; i++ {
		session(i, replicaOutput, sp, replicaID)
	}

	fmt.Printf("\n--------------------------------------\n")
	fmt.Printf("|===> WORKFLOW DONE SUCCESSFULLY <===|\n\n")
	fmt.Printf("--------------------------------------\n")
}
