package oracle

import (
	"strconv"
)

// In a real scenario, this would bridge to an external source or verifiable mechanism.
// For the demo, we simulate deterministic inference results based on input.

var inferredResults []string

// SimulateInference simulates the Oracle Layer deterministically inferring a result
// for an ARKHE Substrate and returning a pseudo-verifiable string.
func SimulateInference(substrateID string, energyKev float64, count int) string {
	// A simple deterministic logic for demonstration
	var class string
	if energyKev < 1000.0 {
		class = "SOLAR"
	} else if energyKev < 1000000.0 {
		class = "ATMOSPHERIC"
	} else {
		class = "EXTRAGALACTIC"
	}

	result := "Substrate:" + substrateID + "-Class:" + class + "-Count:" + strconv.Itoa(count)
	inferredResults = append(inferredResults, result)
	return result
}

// GetRecentInferences returns the recent inference results.
func GetRecentInferences() []string {
	return inferredResults
}

// Render returns a visual representation of the oracle state.
func Render(path string) string {
	if len(inferredResults) == 0 {
		return "No inferences recorded."
	}
	out := "# Oracle Inferences\n\n"
	for _, res := range inferredResults {
		out += "- " + res + "\n"
	}
	return out
}
