package wrapper

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"os"
	"path"
	"testing"

	"github.com/filecoin-project/go-state-types/abi"
	"github.com/stretchr/testify/require"
)

const (
	EX1PIECE1SIZE = 1000
	EX1PIECE2SIZE = 900
	EX1PIECE3SIZE = 255

	EX2PIECESIZE = 2032

	EX3PIECESIZE = 1000

	PIECENAMEPATTERN = "%s_%d.dat"
)

func getTestPieceName(dir, prefix string, size uint64) string {
	return path.Join(dir, fmt.Sprintf(PIECENAMEPATTERN, prefix, size))
}

func createTestPieces(t *testing.T, dir string) {
	require.NoError(t, os.Mkdir(dir, 0666))

	// create pieces for example cases
	// example case 1: 1000B + 900B + 255B
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex1", EX1PIECE1SIZE), EX1PIECE1SIZE))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex1", EX1PIECE2SIZE), EX1PIECE2SIZE))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex1", EX1PIECE3SIZE), EX1PIECE3SIZE))

	// example case 2: 2032
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex2", EX2PIECESIZE), EX2PIECESIZE))

	// example case 3: 1000
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3", EX3PIECESIZE), EX3PIECESIZE))
}

func clearTestPieces(t *testing.T, dir string) {
	require.NoError(t, os.RemoveAll(dir))
}

func sprintPieceInfo(info abi.PieceInfo) string {
	return fmt.Sprintf("{ Size: %dB, CID: %s }", uint64(info.Size), info.PieceCID.String())
}

func sprintCutDetail(cd *CutDetail) string {
	content, err := json.Marshal(cd)
	if err != nil {
		panic(err)
	}

	return string(content)
}

func runMinerAssemblePieces(
	t *testing.T,
	dir string, pieces []string,
	expectedLeft uint64,
) {
	require.NoError(t, os.Mkdir(dir, fakeDataFileMode))
	defer require.NoError(t, os.RemoveAll(dir))

	miner, err := NewMiner(rand.Uint64(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	staged, _, _, err := miner.InitSectorDir(dir)
	defer require.NoError(t, err)

	left, cd, pi, err := miner.AssemblePieces(staged, pieces)
	require.NoError(t, err)
	require.Equal(t, expectedLeft, uint64(left))

	t.Logf("runMinerAssemblePieces(%s): %s\n", dir, sprintCutDetail(cd))
	for _, iter := range pi {
		t.Logf("runMinerAssemblePieces(%s): %s\n", dir, sprintPieceInfo(iter))
	}
}

func TestAssemblePiecesExample1(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex1", EX1PIECE1SIZE),
		getTestPieceName("./ExamplePieces", "ex1", EX1PIECE2SIZE),
		getTestPieceName("./ExamplePieces", "ex1", EX1PIECE3SIZE),
	}
	runMinerAssemblePieces(t, "./AssemblePiecesExample1", pieces, 0)
}
