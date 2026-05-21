package temporalchain

import (
	"testing"
)

func TestAnchorSeal(t *testing.T) {
	chain := NewChain()

	seal := &MsgAnchorSeal{
		SubstrateID: 416,
		PhiC:        0.976,
		SealHash:    "d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5",
		Architect:   "0000-0002-1825-0097",
		Timestamp:   1715424000,
	}

	chain.AnchorSeal(seal)

	seals := chain.GetSeals()
	if len(seals) != 1 {
		t.Fatalf("expected 1 seal, got %d", len(seals))
	}

	if seals[0].SubstrateID != 416 {
		t.Errorf("expected SubstrateID 416, got %d", seals[0].SubstrateID)
	}
	if seals[0].PhiC != 0.976 {
		t.Errorf("expected PhiC 0.976, got %f", seals[0].PhiC)
	}
	if seals[0].SealHash != "d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5" {
		t.Errorf("expected SealHash d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5, got %s", seals[0].SealHash)
	}
}
