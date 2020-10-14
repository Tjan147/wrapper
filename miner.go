package wrapper

// miner demo
//
// 1. build a sector
// 2. create a statement for this sector
// 3. prove while being challenged

import (
	"os"
	"path"

	ffi "github.com/filecoin-project/filecoin-ffi"
	"github.com/filecoin-project/go-state-types/abi"
	"github.com/ipfs/go-cid"
)

const (
	DEFAULTSECTORNAME = "sector.dat"
	DEFAULTSECTORMODE = 0644

	DEFAULTSTAGEDNAME = "staged.dat"
	DEFAULTSTAGEDMODE = 0644

	DEFAULTCACHENAME = "cache"
	DEFAULTCACHEMODE = 0644
)

// Miner is the `storage` part in this demo
type Miner struct {
	ID        abi.ActorID
	ProofType abi.RegisteredSealProof
}

// NewMiner as the factory
func NewMiner(givenID uint64, givenType abi.RegisteredSealProof) Miner {
	return Miner{
		ID:        abi.ActorID(givenID),
		ProofType: givenType,
	}
}

// CreateStagedSector will:
// 1. create a staged file to assemble the data pieces
// 2. create a sector file
// 3. create a cache dir for setup operation
func (m *Miner) CreateStagedSector(p string) (staged *os.File, sector, cache string, err error) {
	sector = path.Join(p, DEFAULTSECTORNAME)
	file, err := os.OpenFile(sector, os.O_CREATE|os.O_RDWR, DEFAULTSECTORMODE)
	if err != nil {
		return
	}
	defer file.Close()

	stagedPath := path.Join(p, DEFAULTSTAGEDNAME)
	staged, err = os.OpenFile(stagedPath, os.O_CREATE|os.O_RDWR, DEFAULTSTAGEDMODE)
	if err != nil {
		os.Remove(sector)

		return
	}

	cache = path.Join(p, DEFAULTCACHENAME)
	if err = os.Mkdir(cache, DEFAULTCACHEMODE); err != nil {
		staged.Close()
		os.Remove(stagedPath)
		os.Remove(sector)

		return
	}

	return
}

// FilPiece tries to assemble piece data to the staged sector file
func (m *Miner) FilPiece(
	pieceFile *os.File,
	pieceLen abi.UnpaddedPieceSize,
	stagedFile *os.File,
	existingPieces []abi.UnpaddedPieceSize,
) (left, total abi.UnpaddedPieceSize, contentID cid.Cid, callErr error) {
	if len(existingPieces) == 0 {
		// write an empty sector
		total, contentID, callErr = ffi.WriteWithoutAlignment(
			m.ProofType,
			pieceFile,
			pieceLen,
			stagedFile,
		)
	} else {
		// non-first write
		left, total, contentID, callErr = ffi.WriteWithAlignment(
			m.ProofType,
			pieceFile,
			pieceLen,
			stagedFile,
			existingPieces,
		)
	}

	return
}

// PoRepSetup actually create the sealed sector file
func (m *Miner) PoRepSetup(
	cacheDir, stagedFilePath, sealedFilePath string,
	sectorNum abi.SectorNumber,
	statementID abi.SealRandomness, // miner generated randomness
	pieces []abi.PieceInfo,
) (sealedCID, unsealedCID cid.Cid, callErr error) {
	preCommitPhase1Output, callErr := ffi.SealPreCommitPhase1(
		m.ProofType,
		cacheDir, stagedFilePath, sealedFilePath,
		sectorNum,
		m.ID,
		statementID,
		pieces,
	)
	if callErr != nil {
		sealedCID = cid.Undef
		unsealedCID = cid.Undef

		return
	}

	sealedCID, unsealedCID, callErr = ffi.SealPreCommitPhase2(
		preCommitPhase1Output,
		cacheDir,
		sealedFilePath,
	)

	return
}

// PoRepProve responses to a PoRep challenge
func (m *Miner) PoRepProve(
	sealedCID, unsealedCID cid.Cid,
	cacheDir, sealedFilePath string,
	sectorNum abi.SectorNumber,
	statementID abi.SealRandomness,
	chal abi.InteractiveSealRandomness,
	pieces []abi.PieceInfo,
) ([]byte, error) {
	commitPhase1Output, callErr := ffi.SealCommitPhase1(
		m.ProofType,
		sealedCID, unsealedCID,
		cacheDir, sealedFilePath,
		sectorNum,
		m.ID,
		statementID,
		chal,
		pieces,
	)
	if callErr != nil {
		return nil, callErr
	}

	return ffi.SealCommitPhase2(commitPhase1Output, sectorNum, m.ID)
}
