package main

import (
	"fmt"
	"math/rand"
	"os"
	"path"
	"strings"
	"time"

	"github.com/filecoin-project/go-state-types/abi"
	"github.com/tjan147/wrapper"
)

func getRandStatementID() abi.SealRandomness {
	ret := make([]byte, 32)
	if _, err := rand.Read(ret); err != nil {
		panic(err)
	}

	return abi.SealRandomness(ret)
}

func inputToProofType(input string) abi.RegisteredSealProof {
	switch strings.ToUpper(strings.TrimSpace(input)) {
	case "2K":
		return abi.RegisteredSealProof_StackedDrg2KiBV1
	case "8M":
		return abi.RegisteredSealProof_StackedDrg8MiBV1
	case "512M":
		return abi.RegisteredSealProof_StackedDrg512MiBV1
	case "32G":
		return abi.RegisteredSealProof_StackedDrg32GiBV1
	}

	fmt.Printf("Unknown sector size %s, replaced with 2K as input\n", input)
	return abi.RegisteredSealProof_StackedDrg2KiBV1
}

func main() {
	// validate the input parameters
	if len(os.Args) != 3 {
		fmt.Println("Require 2 arguments as input parameter.")
		fmt.Println("Example:")
		fmt.Printf("\t%s sample 2k\n", os.Args[0])
		os.Exit(0)
	}
	dir := strings.TrimSpace(os.Args[1])
	typ := inputToProofType(os.Args[2])

	// seed the randomizer
	rand.Seed(time.Now().UnixNano())

	// initialize for the PoRep process
	typSize, err := typ.SectorSize()
	if err != nil {
		panic(err)
	}

	if _, err := os.Stat(dir); !os.IsNotExist(err) {
		os.RemoveAll(dir)
	}
	if err := os.Mkdir(dir, 0755); err != nil {
		panic(err)
	}

	fakePiece := path.Join(dir, "fakepiece.dat")
	if err := wrapper.CreateFakeDataFile(fakePiece, uint64(wrapper.UnpaddedSpace(uint64(typSize)))); err != nil {
		panic(err)
	}

	// create the report instance
	report, err := wrapper.NewReport(dir, typ)
	if err != nil {
		panic(err)
	}

	// prepare the roles
	validator := wrapper.NewValidator()
	miner, err := wrapper.NewMiner(rand.Int63(), typ)
	if err != nil {
		panic(err)
	}
	// MINER pledges to the VALIDATOR
	miner.Pledge(validator)

	// assemble
	step := wrapper.NewStepMeasure("Assemble")
	staged, _, _, err := miner.InitSectorDir(dir)
	if err != nil {
		panic(err)
	}
	_, _, pieceInfos, err := miner.AssemblePieces(staged, []string{fakePiece})
	if err != nil {
		panic(err)
	}
	staged.Close()
	report.AddStep(step.Done())

	// setup
	step = wrapper.NewStepMeasure("Setup")
	statement := miner.CommitStatement(
		getRandStatementID(),
		rand.Uint64(),
		dir,
		pieceInfos,
	)
	report.AddStep(step.Done())

	// challenge
	// the computation here is too simple to be measured
	// MINER post the statement to validator and trigger the handler logic
	validator.HandlePoRepStatement(statement)
	// VALIDATOR generate challenge responding to the commited statement
	validator.GenChallenge()

	// prove
	step = wrapper.NewStepMeasure("Prove")
	challenge := miner.QueryChallengeSet()
	proof := miner.ResponseToChallenge(challenge)
	report.AddStep(step.Done())

	// verify
	step = wrapper.NewStepMeasure("Verify")
	isValid, err := validator.HandlePoRepProof(proof)
	if err != nil {
		panic(err)
	}
	if !isValid {
		panic(fmt.Errorf("porep verification failed"))
	}
	report.AddStep(step.Done())

	// dump report
	if err := report.Dump(); err != nil {
		panic(err)
	}
}
