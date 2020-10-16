package wrapper

// miner demo
//
// 1. build a sector
// 2. create a statement for this sector
// 3. prove while being challenged

import (
	"encoding/base64"
	"fmt"
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

type MinerStorage struct {
	fileDB      map[string]string
	statementDB map[string]*Statement
}

// NewMinerStorage as the factory
func NewMinerStorage() *MinerStorage {
	return &MinerStorage{
		fileDB:      make(map[string]string),
		statementDB: make(map[string]*Statement),
	}
}

func (ms *MinerStorage) SetStatement(st *Statement) {
	key := base64.StdEncoding.EncodeToString([]byte(st.ID))
	ms.statementDB[key] = st
}

func (ms *MinerStorage) GetStatement(givenID abi.SealRandomness) *Statement {
	key := base64.StdEncoding.EncodeToString([]byte(givenID))
	return ms.statementDB[key]
}

func (ms *MinerStorage) SetStatementDir(givenID []byte, path string) {
	key := base64.StdEncoding.EncodeToString(givenID)
	ms.fileDB[key] = path
}

func (ms *MinerStorage) GetStatementDir(givenID abi.SealRandomness) string {
	key := base64.StdEncoding.EncodeToString([]byte(givenID))
	return ms.fileDB[key]
}

// Miner is the `storage` part in this demo
type Miner struct {
	ID                  abi.ActorID
	ProofType           abi.RegisteredSealProof
	SectorUnpaddedSpace abi.UnpaddedPieceSize
	Validator           *Validator
	Store               *MinerStorage
}

// NewMiner as the factory
func NewMiner(givenID int64, givenType abi.RegisteredSealProof) (*Miner, error) {
	sectorSize, err := givenType.SectorSize()
	if err != nil {
		return nil, err
	}
	unpaddedSpace := UnpaddedSpace(uint64(sectorSize))

	return &Miner{
		ID:                  abi.ActorID(givenID),
		ProofType:           givenType,
		SectorUnpaddedSpace: unpaddedSpace,
		Store:               NewMinerStorage(),
	}, nil
}

// RegisterValidator
func (m *Miner) RegisterValidator(val *Validator) {
	m.Validator = val
}

func getSectorName(dir string) string {
	return path.Join(dir, DEFAULTSECTORNAME)
}

func getStagedName(dir string) string {
	return path.Join(dir, DEFAULTSTAGEDNAME)
}

func getCacheName(dir string) string {
	return path.Join(dir, DEFAULTCACHENAME)
}

// InitSectorDir will:
// 1. create a staged file to assemble the data pieces
// 2. create a sector file
// 3. create a cache dir for setup operation
func (m *Miner) InitSectorDir(dir string) (staged *os.File, sector, cache string, err error) {
	sector = getSectorName(dir)
	file, err := os.OpenFile(sector, os.O_CREATE|os.O_RDWR, DEFAULTSECTORMODE)
	if err != nil {
		return
	}
	file.Close()

	stagedPath := getStagedName(dir)
	staged, err = os.OpenFile(stagedPath, os.O_CREATE|os.O_RDWR, DEFAULTSTAGEDMODE)
	if err != nil {
		os.Remove(sector)

		return
	}

	cache = getCacheName(dir)
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

// CommitStatement used to build a statement and send it to validator
func (m *Miner) CommitStatement(
	givenID []byte,
	sectorNum uint64,
	sectorDir string,
	sectorPieces []abi.PieceInfo,
) *Statement {
	// generate
	sealedCID, unsealedCID, err := m.PoRepSetup(
		getCacheName(sectorDir),
		getStagedName(sectorDir),
		getSectorName(sectorDir),
		abi.SectorNumber(sectorNum),
		abi.SealRandomness(givenID),
		sectorPieces,
	)
	if err != nil {
		panic(fmt.Errorf("error generate statement for %s: %s", sectorDir, err))
	}
	m.Store.SetStatementDir(givenID, sectorDir)

	statement := &Statement{
		ID:          abi.SealRandomness(givenID),
		MinerID:     m.ID,
		SectorNum:   abi.SectorNumber(sectorNum),
		SealedCID:   sealedCID,
		UnsealedCID: unsealedCID,
		// optional
		Pieces: sectorPieces,
	}
	m.Store.SetStatement(statement)

	return statement
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

// AnswerChallenge create an answer proof
func (m *Miner) AnswerChallenge(chal *Challenge) *Proof {
	st := m.Store.GetStatement(chal.StatementID)
	sectorDir := m.Store.GetStatementDir(chal.StatementID)

	prf, err := m.PoRepProve(
		st.SealedCID,
		st.UnsealedCID,
		getCacheName(sectorDir),
		getSectorName(sectorDir),
		st.SectorNum,
		st.ID,
		chal.Content,
		st.Pieces,
	)
	if err != nil {
		panic(fmt.Errorf("answer challenge to %s: %s", sectorDir, err))
	}

	return &Proof{
		Content: prf,
	}
}
