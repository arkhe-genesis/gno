package temporalchain

import (
	"testing"
)

func TestCalculateMerkleRoot(t *testing.T) {
	items := []string{"item1", "item2", "item3"}
	root := CalculateMerkleRoot(items)
	if root == "" {
		t.Errorf("expected a valid hash, got empty string")
	}

	rootEmpty := CalculateMerkleRoot([]string{})
	if rootEmpty != "" {
		t.Errorf("expected empty string for empty items, got %s", rootEmpty)
	}
}

func TestChain(t *testing.T) {
	chain := NewChain()
	block := &ThetaT0Block{
		ID:        "T0-1",
		MerkleRoot: "mock-root",
		Timestamp: 1620000000,
		Metadata:  "Genesis block",
	}
	chain.AnchorBlock(block)

	blocks := chain.GetBlocks()
	if len(blocks) != 1 {
		t.Fatalf("expected 1 block, got %d", len(blocks))
	}

	if blocks[0].ID != "T0-1" {
		t.Errorf("expected block ID T0-1, got %s", blocks[0].ID)
	}
}
