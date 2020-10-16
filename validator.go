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

type ValidatorKeeper struct {
	statement *Statement
}

// Validator is the statement challenge
type Validator struct {
	store *Statement
}

// RANDBUFLEN is the length of random bytes
const RANDBUFLEN = 32

func (v *Validator) handlePoRepStatement(st *Statement) {
}

func (v *Validator) GenChallenge() {
}

// PoRepChallenge fire a challenge
func (v *Validator) PoRepChallenge() abi.InteractiveSealRandomness {
	ret := make([]byte, RANDBUFLEN)
	if _, err := rand.Read(ret); err != nil {
		panic(err)
	}

	return abi.InteractiveSealRandomness(ret)
}

func (v *Validator) QueryChallengeSet() *Challenge {
	return nil
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

func (v *Validator) handlePoRepProof(prf *Proof) (bool, error) {

}
