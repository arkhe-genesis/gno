package temporalchain

import (
	"crypto/sha256"
	"encoding/hex"
	"strings"
)

// ThetaT0Block represents a temporal anchor block in the TemporalChain.
type ThetaT0Block struct {
	ID        string
	MerkleRoot string
	Timestamp int64
	Metadata  string
}

// Chain is the in-memory representation of the TemporalChain for this package/realm.
type Chain struct {
	blocks []*ThetaT0Block
}

// NewChain creates a new TemporalChain.
func NewChain() *Chain {
	return &Chain{
		blocks: make([]*ThetaT0Block, 0),
	}
}

// AnchorBlock adds a new Theta-T0 block to the TemporalChain.
func (c *Chain) AnchorBlock(block *ThetaT0Block) {
	c.blocks = append(c.blocks, block)
}

// GetBlocks returns all anchored blocks in the TemporalChain.
func (c *Chain) GetBlocks() []*ThetaT0Block {
	return c.blocks
}

// CalculateMerkleRoot computes a simple pseudo-Merkle root from a list of strings (data hashes/items).
// For simplicity in Gno, we hash the concatenated list of items.
func CalculateMerkleRoot(items []string) string {
	if len(items) == 0 {
		return ""
	}

	// A real Merkle root would build a tree. For this demo, we do a simplistic cumulative hash.
	concatenated := strings.Join(items, "")
	hash := sha256.Sum256([]byte(concatenated))
	return hex.EncodeToString(hash[:])
}

// GenerateSeal produces a SHA-256 seal for arbitrary data
func GenerateSeal(data string) string {
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}
