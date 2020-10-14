package wrapper

// validator demo
//
// 1. challenge the statemnet provided by miner
// 2. verify the proof

import (
	"crypto/rand"

	ffi "github.com/filecoin-project/filecoin-ffi"
	"github.com/filecoin-project/go-state-types/abi"
	prf "github.com/filecoin-project/specs-actors/actors/runtime/proof"
	"github.com/ipfs/go-cid"
)

// Validator is the statement challenge
type Validator struct {
	storeMiner     *Miner
	storeStatement *Statement
	challengeSet   abi.InteractiveSealRandomness
}

// PoRepChallenge fire a challenge
func (v *Validator) PoRepChallenge() abi.InteractiveSealRandomness {
	ret := make([]byte, 16)
	if _, err := rand.Read(ret); err != nil {
		panic(err)
	}

	return ret
}

// PoRepVerify validate the proof commit by miner
func (v *Validator) PoRepVerify(
	minerID abi.ActorID,
	sectorNum abi.SectorNumber,
	sealedCID, unsealedCID cid.Cid,
	statementID abi.SealRandomness,
	chal abi.InteractiveSealRandomness,
	proof []byte,
) (bool, error) {
	return ffi.VerifySeal(prf.SealVerifyInfo{
		SectorID: abi.SectorID{
			Miner:  minerID,
			Number: sectorNum,
		},
		SealedCID:             sealedCID,
		SealProof:             v.storeMiner.ProofType,
		Proof:                 proof,
		DealIDs:               []abi.DealID{},
		Randomness:            statementID,
		InteractiveRandomness: chal,
		UnsealedCID:           unsealedCID,
	})
}
