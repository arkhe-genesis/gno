package oracle

import (
	"testing"
)

func TestSimulateInference(t *testing.T) {
	result := SimulateInference("412-SRC", 500.0, 10)
	expected := "Substrate:412-SRC-Class:SOLAR-Count:10"
	if result != expected {
		t.Errorf("expected %s, got %s", expected, result)
	}

	results := GetRecentInferences()
	if len(results) != 1 {
		t.Errorf("expected 1 result, got %d", len(results))
	}
}
