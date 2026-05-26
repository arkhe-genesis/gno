package temporalanchor

import (
	"strconv"
	"gno.land/p/arkhe/temporalchain"
)

var chain *temporalchain.Chain

func init() {
	chain = temporalchain.NewChain()
}

// AnchorSubstrate takes a substrate ID and associated metadata/hashes,
// generates an auto-Merkle proof using temporalchain package, and anchors it.
func AnchorSubstrate(substrateID string, hashes []string, timestamp int64) string {
	merkleRoot := temporalchain.CalculateMerkleRoot(hashes)

	block := &temporalchain.ThetaT0Block{
		ID:        substrateID,
		MerkleRoot: merkleRoot,
		Timestamp: timestamp,
		Metadata:  "Auto-Merkle Proof Anchored",
	}

	chain.AnchorBlock(block)
	return "Anchored block with Merkle Root: " + merkleRoot
}

// Render returns a string representation of all anchored Theta-T0 blocks.
func Render(path string) string {
	blocks := chain.GetBlocks()
	if len(blocks) == 0 {
		return "No Theta-T0 blocks anchored yet."
	}

	out := "# Temporal Anchors (Theta-T0)\n\n"
	for _, b := range blocks {
		out += "## Block ID: " + b.ID + "\n"
		out += "- Merkle Root: " + b.MerkleRoot + "\n"
		out += "- Timestamp: " + strconv.FormatInt(b.Timestamp, 10) + "\n"
		out += "- Metadata: " + b.Metadata + "\n\n"
	}
	return out
}
