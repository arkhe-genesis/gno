package invariants

import (
	"testing"
	"gno.land/p/demo/temporalchain"
)

func TestVerifyAndAnchor(t *testing.T) {
	// Re-initialize chain for testing manually since init() is called automatically
	chain = temporalchain.NewChain()

	validData := &SubstrateData{
		ID:                416,
		HasContradiction:  false,
		IsLoopClosed:      true,
		HasDocumentedGaps: true,
		PhiCRatio:         0.976,
		SealHash:          "d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5",
		Architect:         "0000-0002-1825-0097",
		Timestamp:         1715424000,
	}

	result := VerifyAndAnchor(validData)
	if result != "CANONIZED: Substrate anchored successfully." {
		t.Fatalf("expected canonization, got %s", result)
	}

	invalidData := &SubstrateData{
		ID:                417,
		HasContradiction:  true,
		IsLoopClosed:      true,
		HasDocumentedGaps: true,
		PhiCRatio:         0.976,
		SealHash:          "dummy",
		Architect:         "dummy",
		Timestamp:         1715424000,
	}

	resultInvalid := VerifyAndAnchor(invalidData)
	if resultInvalid != "REJECTED: Ghost Contract violated (internal contradiction detected)." {
		t.Fatalf("expected ghost violation, got %s", resultInvalid)
	}
}
