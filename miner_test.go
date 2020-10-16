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

var (
	EX1PIECE1SIZE = UnpaddedSpace(1024)
	EX1PIECE2SIZE = UnpaddedSpace(512)
	EX1PIECE3SIZE = UnpaddedSpace(256)

	EX2PIECESIZE = UnpaddedSpace(2048)

	EX3PIECE1SIZE = UnpaddedSpace(128)
	EX3PIECE2SIZE = UnpaddedSpace(256)
	EX3PIECE3SIZE = UnpaddedSpace(512)
	EX3PIECE4SIZE = UnpaddedSpace(1024)
	EX3PIECE5SIZE = UnpaddedSpace(2048)
	EX3PIECE6SIZE = UnpaddedSpace(4096)
	EX3PIECE7SIZE = UnpaddedSpace(8192)

	PIECENAMEPATTERN = "%s_%d.dat"
)

func getTestPieceName(dir, prefix string, size uint64) string {
	return path.Join(dir, fmt.Sprintf(PIECENAMEPATTERN, prefix, size))
}

func createTestPieces(t *testing.T, dir string) {
	require.NoError(t, os.Mkdir(dir, 0755))

	// create pieces for example cases
	// example case 1: 255B + 900B + 1023B
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex1_1", uint64(EX1PIECE1SIZE)), uint64(EX1PIECE1SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex1_2", uint64(EX1PIECE2SIZE)), uint64(EX1PIECE2SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex1_3", uint64(EX1PIECE3SIZE)), uint64(EX1PIECE3SIZE)))

	// example case 2: 2032
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex2", uint64(EX2PIECESIZE)), uint64(EX2PIECESIZE)))

	// example case 3: 1000
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_1", uint64(EX3PIECE1SIZE)), uint64(EX3PIECE1SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_2", uint64(EX3PIECE2SIZE)), uint64(EX3PIECE2SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_3", uint64(EX3PIECE3SIZE)), uint64(EX3PIECE3SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_4", uint64(EX3PIECE4SIZE)), uint64(EX3PIECE4SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_5", uint64(EX3PIECE5SIZE)), uint64(EX3PIECE5SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_6", uint64(EX3PIECE6SIZE)), uint64(EX3PIECE6SIZE)))
	require.NoError(t, CreateFakeDataFile(getTestPieceName(dir, "ex3_7", uint64(EX3PIECE7SIZE)), uint64(EX3PIECE7SIZE)))
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
	typ abi.RegisteredSealProof,
	dir string, pieces []string,
	expectedLeft uint64,
) {
	require.NoError(t, os.Mkdir(dir, 0755))
	defer require.NoError(t, os.RemoveAll(dir))

	miner, err := NewMiner(rand.Uint64(), typ)

	staged, _, _, err := miner.InitSectorDir(dir)
	defer require.NoError(t, err)

	left, total, pi, err := miner.AssemblePieces(staged, pieces)
	require.NoError(t, err)

	t.Logf("runMinerAssemblePieces(%s): %d, %d\n", dir, left, total)
	for _, iter := range pi {
		t.Logf("runMinerAssemblePieces(%s): %s\n", dir, sprintPieceInfo(iter))
	}
}

func TestAssemblePiecesExample1(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex1_1", uint64(EX1PIECE1SIZE)),
		getTestPieceName("./ExamplePieces", "ex1_2", uint64(EX1PIECE2SIZE)),
		getTestPieceName("./ExamplePieces", "ex1_3", uint64(EX1PIECE3SIZE)),
	}
	runMinerAssemblePieces(t, abi.RegisteredSealProof_StackedDrg2KiBV1, "./AssemblePiecesExample1", pieces, 0)
}

func TestAssemblePiecesExample2(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex2", uint64(EX2PIECESIZE)),
	}
	runMinerAssemblePieces(t, abi.RegisteredSealProof_StackedDrg2KiBV1, "./AssemblePiecesExample2", pieces, 0)
}

func TestAssemblePiecesExample3(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex3_1", uint64(EX3PIECE1SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_2", uint64(EX3PIECE2SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_3", uint64(EX3PIECE3SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_4", uint64(EX3PIECE4SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_5", uint64(EX3PIECE5SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_6", uint64(EX3PIECE6SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_7", uint64(EX3PIECE7SIZE)),
	}
	runMinerAssemblePieces(t, abi.RegisteredSealProof_StackedDrg8MiBV1, "./AssemblePiecesExample3", pieces, 0)
}
