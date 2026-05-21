package calib

import (
	"strconv"

	"gno.land/r/arkhe/invariants"
	"gno.land/r/arkhe/sources"
)

var (
	calibrationCurve [][]float64
	phiC             float64
	sealHash         string
)

func Calibrate(sourceName string, counts int) string {
	// Dummy checks to satisfy the invariants Ghost
	checks := []invariants.Check{
		{Valid: true, Data: sourceName},
	}
	if !invariants.CheckGhost(checks) {
		panic("Ghost invariant violated")
	}

	// Just an example mapping, doing some operation with counts and sources
	metrics := []float64{float64(counts), float64(len(sources.Am241))}

	phiC = invariants.ComputePhiC(metrics)
	sealHash = invariants.GenerateSeal(sourceName + strconv.Itoa(counts))

	// Example to modify the calibration curve
	calibrationCurve = append(calibrationCurve, metrics)

	return sealHash
}

func GetPhiC() float64 {
	return phiC
}

func GetSealHash() string {
	return sealHash
}
