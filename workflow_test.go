package wrapper

import (
	"math/rand"
	"os"
	"testing"
	"time"

	"github.com/filecoin-project/go-state-types/abi"
	"github.com/stretchr/testify/require"
)

func userUploadPieces(t *testing.T, dir string) []string {
	rand.Seed(time.Now().UnixNano())

	require.NoError(t, os.Mkdir(dir, 0755))

	// create pieces for example cases
	// example case 1: 255B + 900B + 1023B
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "user_1", uint64(EX1PIECE1SIZE)), uint64(EX1PIECE1SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "user_2", uint64(EX1PIECE2SIZE)), uint64(EX1PIECE2SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "user_3", uint64(EX1PIECE3SIZE)), uint64(EX1PIECE3SIZE)))

	return []string{
		getTestPieceName(dir, "user_1", uint64(EX1PIECE1SIZE)),
		getTestPieceName(dir, "user_2", uint64(EX1PIECE2SIZE)),
		getTestPieceName(dir, "user_3", uint64(EX1PIECE3SIZE)),
	}
}

func TestWorkflow(t *testing.T) {
	rand.Seed(time.Now().UnixNano())

	// Roles
	validator := NewValidator()
	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	// MINER pledges to the VALIDATOR
	miner.Pledge(validator)

	// MINER receive data pieces from USER
	sampleDir := "./workflow"
	pieces := userUploadPieces(t, sampleDir)

	// MINER assemble the pieces as staged file
	staged, _, _, err := miner.InitSectorDir(sampleDir)
	require.NoError(t, err)

	_, _, pieceInfos, err := miner.AssemblePieces(staged, pieces)
	require.NoError(t, err)
	require.NoError(t, staged.Close())

	// MINER apply the PoRep setup upon the staged data
	// and form a statement
	start := time.Now()
	statement := miner.CommitStatement(
		getRandStatementID(),
		uint64(getRandSectorNum()),
		sampleDir,
		pieceInfos,
	)
	t.Logf("2k-sector PoRep setup takes %s ...\n", time.Now().Sub(start).String())
	t.Logf("2k-sector PoRep setup returns sealed CID: %s\n", statement.SealedCID.String())
	t.Logf("2k-sector PoRep setup returns unsealed CID: %s\n", statement.UnsealedCID.String())

	// MINER post the statement to validator and trigger the handler logic
	validator.HandlePoRepStatement(statement)

	// VALIDATOR generate challenge responding to the commited statement
	validator.GenChallenge()

	// MINER query the validator for challenge infomation & response to the challenge
	challenge := miner.QueryChallengeSet()
	start = time.Now()
	proof := miner.ResponseToChallenge(challenge)
	t.Logf("2k-sector PoRep prove takes %s...\n", time.Now().Sub(start).String())
	t.Logf("2k-sector PoRep prove returns %dB-lenght proof\n", len(proof.Content))

	// VALIDATOR tries to verify the proof commited by MINER
	start = time.Now()
	isValid, err := validator.HandlePoRepProof(proof)
	t.Logf("2k-sector PoRep verify takes %s...\n", time.Now().Sub(start).String())
	require.NoError(t, err)
	require.True(t, isValid)

	require.NoError(t, os.RemoveAll(sampleDir))
}
