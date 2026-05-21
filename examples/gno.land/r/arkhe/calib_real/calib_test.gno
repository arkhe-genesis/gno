package calib

import (
	"testing"
	"gno.land/r/arkhe/sources"
)

func TestCalibrate(t *testing.T) {
	seal := Calibrate(sources.Am241, 100)
	if seal == "" {
		t.Errorf("Expected non-empty seal, got empty string")
	}

	phiC := GetPhiC()
	if phiC != 53.0 {
		t.Errorf("Expected phiC to be 53.0, got %f", phiC)
	}
}
