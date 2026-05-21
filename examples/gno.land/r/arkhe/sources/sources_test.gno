package sources

import "testing"

func TestSources(t *testing.T) {
	if Am241 == "" {
		t.Errorf("Expected Am241 not empty")
	}
}
