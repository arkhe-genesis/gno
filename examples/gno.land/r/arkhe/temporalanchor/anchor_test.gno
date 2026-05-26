package temporalanchor

import (
	"strings"
	"testing"
)

func TestAnchorSubstrate(t *testing.T) {
	hashes := []string{"hash1", "hash2"}
	result := AnchorSubstrate("Substrate-412", hashes, 1620000000)

	if !strings.Contains(result, "Anchored block with Merkle Root:") {
		t.Errorf("unexpected result: %s", result)
	}

	renderOut := Render("")
	if !strings.Contains(renderOut, "Substrate-412") {
		t.Errorf("Render output missing anchored block: %s", renderOut)
	}
}
