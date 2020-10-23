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

type Keeper struct {
	statement *Statement
	challenge *Challenge
}

// SetStatement save the validator commited statement
func (k *Keeper) SetStatement(st *Statement) {
	k.statement = st
}

// GetStatement as getter
func (k *Keeper) GetStatement() *Statement {
	return k.statement
}

// PickStatement mimic the actual method with the same name
func (k *Keeper) PickStatement() *Statement {
	return k.statement
}

// SetChallenge
func (k *Keeper) SetChallenge(chal *Challenge) {
	k.challenge = chal
}

// GetChallenge
func (k *Keeper) GetChallenge() *Challenge {
	return k.challenge
}

// Validator is the statement challenge
type Validator struct {
	keeper *Keeper
}

// NewValidator as the factor method
func NewValidator() *Validator {
	k := &Keeper{
		statement: nil,
		challenge: nil,
	}

	return &Validator{
		keeper: k,
	}
}

// RANDBUFLEN is the length of random bytes
const RANDBUFLEN = 32

// HandlePoRepStatement mimics the cosmos handler
func (v *Validator) HandlePoRepStatement(st *Statement) {
	v.keeper.SetStatement(st)
}

// PoRepChallenge fire a challenge
func (v *Validator) PoRepChallenge() abi.InteractiveSealRandomness {
	ret := make([]byte, RANDBUFLEN)
	if _, err := rand.Read(ret); err != nil {
		panic(err)
	}

	return abi.InteractiveSealRandomness(ret)
}

// GenChallenge mimic the actual method with the same name
func (v *Validator) GenChallenge() {
	st := v.keeper.PickStatement()
	chal := v.PoRepChallenge()

	v.keeper.SetChallenge(&Challenge{
		StatementID: st.ID,
		Content:     chal,
	})
}

func (v *Validator) queryChallengeSet() *Challenge {
	return v.keeper.GetChallenge()
}

// PoRepVerify validate the proof commit by miner
func (v *Validator) PoRepVerify(
	minerID abi.ActorID,
	sectorNum abi.SectorNumber,
	proofType abi.RegisteredSealProof,
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
		SealProof:             proofType,
		Proof:                 proof,
		DealIDs:               []abi.DealID{},
		Randomness:            statementID,
		InteractiveRandomness: chal,
		UnsealedCID:           unsealedCID,
	})
}

// HandlePoRepProof mimics the cosmos handler
func (v *Validator) HandlePoRepProof(prf *Proof) (bool, error) {
	chal := v.keeper.GetChallenge()
	st := v.keeper.GetStatement()

	return v.PoRepVerify(
		st.MinerID,
		st.SectorNum,
		st.ProofType,
		st.SealedCID,
		st.UnsealedCID,
		st.ID,
		chal.Content,
		prf.Content,
	)
}
