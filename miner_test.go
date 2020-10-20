package wrapper

import (
	"fmt"
	"math/rand"
	"os"
	"path"
	"testing"
	"time"

	"github.com/filecoin-project/go-state-types/abi"
	"github.com/stretchr/testify/require"
)

var (
	// WARNING: all piece size should be integer power of 2 (unpadded)
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
	rand.Seed(time.Now().UnixNano())

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
	return fmt.Sprintf("{ Size: %d, CID: %s }", uint64(info.Size), info.PieceCID.String())
}

func prettySprintPieceInfos(infos []abi.PieceInfo) string {
	ret := "{\n"
	for _, i := range infos {
		ret += fmt.Sprintf("\t%s,\n", sprintPieceInfo(i))
	}
	return ret + "}\n"
}

func getRandSectorNum() abi.SectorNumber {
	return abi.SectorNumber(rand.Uint64())
}

func getRandStatementID() abi.SealRandomness {
	ret := make([]byte, RANDBUFLEN)
	if _, err := rand.Read(ret); err != nil {
		panic(err)
	}

	return abi.SealRandomness(ret)
}

func assemblePieces(
	t *testing.T,
	miner *Miner,
	dir string, pieces []string,
	expectedLeft uint64,
	needClear bool,
) (stagedPath, sectorPath, cachePath string, pieceInfos []abi.PieceInfo) {
	require.NoError(t, os.Mkdir(dir, 0755))

	staged, sectorPath, cachePath, err := miner.InitSectorDir(dir)
	require.NoError(t, err)
	stagedPath = staged.Name()

	_, _, pieceInfos, err = miner.AssemblePieces(staged, pieces)
	require.NoError(t, err)

	stagedMeta, err := staged.Stat()
	require.NoError(t, err)
	sectorSize, err := miner.ProofType.SectorSize()
	require.NoError(t, err)
	t.Logf("assemblePieces(%d): %d, %s", sectorSize, stagedMeta.Size(), prettySprintPieceInfos(pieceInfos))

	require.NoError(t, staged.Close())
	if needClear {
		require.NoError(t, os.RemoveAll(dir))
	}

	return
}

func TestAssemblePiecesExample1(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex1_1", uint64(EX1PIECE1SIZE)),
		getTestPieceName("./ExamplePieces", "ex1_2", uint64(EX1PIECE2SIZE)),
		getTestPieceName("./ExamplePieces", "ex1_3", uint64(EX1PIECE3SIZE)),
	}
	assemblePieces(t, miner, "./AssemblePiecesEx1", pieces, 0, true)
}

func TestAssemblePiecesExample2(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex2", uint64(EX2PIECESIZE)),
	}
	assemblePieces(t, miner, "./AssemblePiecesEx2", pieces, 0, true)
}

func TestAssemblePiecesExample3(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex3_1", uint64(EX3PIECE1SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_2", uint64(EX3PIECE2SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_3", uint64(EX3PIECE3SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_4", uint64(EX3PIECE4SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_5", uint64(EX3PIECE5SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_6", uint64(EX3PIECE6SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_7", uint64(EX3PIECE7SIZE)),
	}
	assemblePieces(t, miner, "./AssemblePiecesEx3", pieces, 0, true)
}

func TestPoRepSetupExample1(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex1_1", uint64(EX1PIECE1SIZE)),
		getTestPieceName("./ExamplePieces", "ex1_2", uint64(EX1PIECE2SIZE)),
		getTestPieceName("./ExamplePieces", "ex1_3", uint64(EX1PIECE3SIZE)),
	}
	staged, sector, cache, pieceInfos := assemblePieces(t, miner, "./SetupEx1", pieces, 0, false)

	start := time.Now()
	sealedCID, unsealedCID, err := miner.PoRepSetup(
		cache, staged, sector,
		getRandSectorNum(),
		getRandStatementID(),
		pieceInfos,
	)
	require.NoError(t, err)
	t.Logf("PoRepSetup() takes %s ...\n", time.Now().Sub(start).String())

	t.Logf("sealedCID = {%s}\n", sealedCID.String())
	t.Logf("unsealedCID = {%s}\n", unsealedCID.String())
	require.NoError(t, os.RemoveAll("./SetupEx1"))
}

func TestPoRepSetupExample2(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg2KiBV1)
	require.NoError(t, err)

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex2", uint64(EX2PIECESIZE)),
	}
	staged, sector, cache, pieceInfos := assemblePieces(t, miner, "./SetupEx2", pieces, 0, false)

	start := time.Now()
	sealedCID, unsealedCID, err := miner.PoRepSetup(
		cache, staged, sector,
		getRandSectorNum(),
		getRandStatementID(),
		pieceInfos,
	)
	require.NoError(t, err)
	t.Logf("PoRepSetup() takes %s ...\n", time.Now().Sub(start).String())

	t.Logf("sealedCID = {%s}\n", sealedCID.String())
	t.Logf("unsealedCID = {%s}\n", unsealedCID.String())
	require.NoError(t, os.RemoveAll("./SetupEx2"))
}

func TestPoRepSetupExample3(t *testing.T) {
	createTestPieces(t, "./ExamplePieces")
	defer clearTestPieces(t, "./ExamplePieces")

	miner, err := NewMiner(rand.Int63(), abi.RegisteredSealProof_StackedDrg8MiBV1)
	require.NoError(t, err)

	pieces := []string{
		getTestPieceName("./ExamplePieces", "ex3_1", uint64(EX3PIECE1SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_2", uint64(EX3PIECE2SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_3", uint64(EX3PIECE3SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_4", uint64(EX3PIECE4SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_5", uint64(EX3PIECE5SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_6", uint64(EX3PIECE6SIZE)),
		getTestPieceName("./ExamplePieces", "ex3_7", uint64(EX3PIECE7SIZE)),
	}
	staged, sector, cache, pieceInfos := assemblePieces(t, miner, "./SetupEx3", pieces, 0, false)

	start := time.Now()
	sealedCID, unsealedCID, err := miner.PoRepSetup(
		cache, staged, sector,
		getRandSectorNum(),
		getRandStatementID(),
		pieceInfos,
	)
	require.Error(t, err)
	t.Logf("PoRepSetup() takes %s ...\n", time.Now().Sub(start).String())
	t.Logf("Oversized staged file cause issue: %s\n", err)

	t.Logf("sealedCID = {%s}\n", sealedCID.String())
	t.Logf("unsealedCID = {%s}\n", unsealedCID.String())
	require.NoError(t, os.RemoveAll("./SetupEx3"))
}
