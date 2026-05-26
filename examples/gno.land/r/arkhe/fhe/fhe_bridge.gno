package fhe

import (
	"time"
)

type Address string

type FheEscrow struct {
	Owner            Address
	CircleID         string
	LockedAmount     uint64
	EncryptedBalance []byte
	Status           string
	CreatedAt        time.Time
	ComputationID    uint64
}

var (
	escrows      = make(map[uint64]*FheEscrow)
	nextEscrowID uint64 = 1
	bridgeAdmin  Address
)

func init() {
}

func CreateEscrow(circleID string, amount uint64, encryptedBalance []byte) uint64 {
	var caller Address = "g1mockaddress"
	escrow := &FheEscrow{
		Owner:            caller,
		CircleID:         circleID,
		LockedAmount:     amount,
		EncryptedBalance: encryptedBalance,
		Status:           "active",
		CreatedAt:        time.Now(),
	}
	id := nextEscrowID
	nextEscrowID++
	escrows[id] = escrow
	return id
}

func TriggerFheCompute(escrowID uint64, circuitID string, gateProgram string) bool {
	escrow, exists := escrows[escrowID]
	if !exists {
		panic("escrow not found")
	}

	escrow.Status = "computing"
	escrow.ComputationID = escrowID

	return true
}

func CompleteFheCompute(escrowID uint64, resultProof []byte, zkProof []byte) bool {
	escrow, exists := escrows[escrowID]
	if !exists {
		panic("escrow not found")
	}

	if escrow.Status != "computing" {
		panic("escrow not computing")
	}
	if !verifyZkProof(resultProof, zkProof) {
		panic("invalid ZK proof")
	}
	escrow.Status = "completed"

	return true
}

func Withdraw(escrowID uint64) bool {
	escrow, exists := escrows[escrowID]
	if !exists {
		panic("escrow not found")
	}

	if escrow.Status != "completed" {
		panic("computation not completed")
	}
	escrow.Status = "withdrawn"

	return true
}

func QueryEscrow(escrowID uint64) (Address, string, string, uint64) {
	escrow, exists := escrows[escrowID]
	if !exists {
		panic("escrow not found")
	}
	return escrow.Owner, escrow.CircleID, escrow.Status, escrow.LockedAmount
}

func verifyZkProof(resultProof []byte, zkProof []byte) bool {
	return len(zkProof) > 32 && len(resultProof) > 0
}

func u64toa(n uint64) string {
	if n == 0 {
		return "0"
	}
	var buf [20]byte
	i := len(buf)
	for n > 0 {
		i--
		buf[i] = byte('0' + n%10)
		n /= 10
	}
	return string(buf[i:])
}
