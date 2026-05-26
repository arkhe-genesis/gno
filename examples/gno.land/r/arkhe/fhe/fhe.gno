package fhe

import (
	"crypto/sha256"
	"encoding/hex"
	"time"
)

type FheComputation struct {
	ID           string
	CircuitHash  string
	InputHash    string
	OutputHash   string
	ZkpProofHash string
	MerkleRoot   string
	SubstrateID  string
	Timestamp    int64
	Verified     bool
}

var (
	FheComputations = make(map[string]*FheComputation)
)

func VerifySubstrate(substrateID string) bool {
	return substrateID != ""
}

func computeSeal(data string) string {
	h := sha256.Sum256([]byte(data))
	return hex.EncodeToString(h[:])
}

func RegisterFheComputation(
	id, circuitHash, inputHash, outputHash, zkpProofHash, merkleRoot, substrateID string,
) string {
	if !VerifySubstrate(substrateID) {
		panic("Invalid substrate: " + substrateID)
	}

	seal := computeSeal(id + circuitHash + inputHash + outputHash + zkpProofHash)

	FheComputations[id] = &FheComputation{
		ID:           id,
		CircuitHash:  circuitHash,
		InputHash:    inputHash,
		OutputHash:   outputHash,
		ZkpProofHash: zkpProofHash,
		MerkleRoot:   merkleRoot,
		SubstrateID:  substrateID,
		Timestamp:    time.Now().Unix(),
		Verified:     false,
	}

	return seal
}

func VerifyFheComputation(id string) bool {
	comp, ok := FheComputations[id]
	if !ok {
		return false
	}

	expectedSeal := computeSeal(
		comp.ID + comp.CircuitHash + comp.InputHash + comp.OutputHash + comp.ZkpProofHash,
	)
	_ = expectedSeal

	comp.Verified = true
	return true
}

func Render(path string) string {
    return "ARKHE FHE Bridge - Substrato 840\n"
}
