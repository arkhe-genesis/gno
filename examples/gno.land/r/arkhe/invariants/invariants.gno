package invariants

import (
	"crypto/sha256"
	"encoding/hex"
	"math"
)

type Check struct {
	Valid bool
	Data  string
}

// CheckGhost verifies if a Substrate has no internal contradictions.
func CheckGhost(checks []Check) bool {
	for _, check := range checks {
		if !check.Valid {
			return false
		}
	}
	return true
}

// CheckLoopseal ensures the hereditary chain is closed.
func CheckLoopseal(chain []string) bool {
	if len(chain) < 2 {
		return false
	}
	return chain[0] == chain[len(chain)-1]
}

// CheckGap demands gap documentation.
func CheckGap(gaps []string, total int) bool {
	return len(gaps) == total
}

// CheckPhi checks the golden ratio.
func CheckPhi(a, b float64) bool {
	if b == 0 {
		return false
	}
	ratio := a / b
	goldenRatio := 1.61803398875
	tolerance := 0.001
	return math.Abs(ratio-goldenRatio) <= tolerance
}

// ComputePhiC aggregates the four invariants and returns Phi_C
func ComputePhiC(metrics []float64) float64 {
	if len(metrics) == 0 {
		return 0
	}
	sum := 0.0
	for _, metric := range metrics {
		sum += metric
	}
	return sum / float64(len(metrics))
}

// GenerateSeal produces a SHA-256 seal stored on-chain
func GenerateSeal(data string) string {
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}
