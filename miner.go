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
	DEFAULTCACHEMODE = 0755
)

// Miner is the `storage` part in this demo
type Miner struct {
	ID                  abi.ActorID
	ProofType           abi.RegisteredSealProof
	SectorUnpaddedSpace abi.UnpaddedPieceSize
}

// NewMiner as the factory
func NewMiner(givenID uint64, givenType abi.RegisteredSealProof) (*Miner, error) {
	sectorSize, err := givenType.SectorSize()
	if err != nil {
		return nil, err
	}
	unpaddedSpace := UnpaddedSpace(uint64(sectorSize))

	return &Miner{
		ID:                  abi.ActorID(givenID),
		ProofType:           givenType,
		SectorUnpaddedSpace: unpaddedSpace,
	}, nil
}

// InitSectorDir will:
// 1. create a staged file to assemble the data pieces
// 2. create a sector file
// 3. create a cache dir for setup operation
func (m *Miner) InitSectorDir(dir string) (staged *os.File, sector, cache string, err error) {
	sector = path.Join(dir, DEFAULTSECTORNAME)
	file, err := os.OpenFile(sector, os.O_CREATE|os.O_RDWR, DEFAULTSECTORMODE)
	if err != nil {
		return
	}
	file.Close()

	stagedPath := path.Join(dir, DEFAULTSTAGEDNAME)
	staged, err = os.OpenFile(stagedPath, os.O_CREATE|os.O_RDWR, DEFAULTSTAGEDMODE)
	if err != nil {
		os.Remove(sector)

		return
	}

	cache = path.Join(dir, DEFAULTCACHENAME)
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

		left -= pieceLen
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

// CutDetail show the stop point of a pieces array
type CutDetail struct {
	Path   string `json:"path"`
	Offset uint64 `json:"Offset"`
}

// AssemblePieces tries to assemble pieces
func (m *Miner) AssemblePieces(staged *os.File, piecePaths []string) (
	left, total abi.UnpaddedPieceSize,
	pi []abi.PieceInfo,
	err error,
) {
	pi = make([]abi.PieceInfo, 0)
	left = m.SectorUnpaddedSpace
	existing := make([]abi.UnpaddedPieceSize, 0)
	for _, p := range piecePaths {
		if left == 0 {
			break
		}

		meta, innerErr := os.Stat(p)
		if innerErr != nil {
			err = innerErr
			return
		}

		piece, innerErr := os.Open(p)
		if innerErr != nil {
			err = innerErr
			return
		}
		defer piece.Close()

		var pieceCID cid.Cid
		left, total, pieceCID, innerErr = m.FilPiece(piece, abi.UnpaddedPieceSize(meta.Size()), staged, existing)
		if innerErr != nil {
			err = innerErr
			return
		}

		pi = append(pi, abi.PieceInfo{
			Size:     abi.UnpaddedPieceSize(meta.Size()).Padded(),
			PieceCID: pieceCID,
		})
		existing = append(existing, abi.UnpaddedPieceSize(meta.Size()))
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
