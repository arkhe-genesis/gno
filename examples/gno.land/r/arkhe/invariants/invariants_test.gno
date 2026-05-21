package invariants

import (
	"testing"
)

func TestCheckGhost(t *testing.T) {
	checks := []Check{
		{Valid: true, Data: "test"},
	}
	if !CheckGhost(checks) {
		t.Errorf("Expected true, got false")
	}
}

func TestCheckPhi(t *testing.T) {
	if !CheckPhi(1.618, 1.0) {
		t.Errorf("Expected true, got false")
	}
	if CheckPhi(2.0, 1.0) {
		t.Errorf("Expected false, got true")
	}
}
