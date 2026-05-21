package invariants

import (
	"gno.land/p/demo/temporalchain"
	"strings"
)

var (
	// chain represents the TemporalChain singleton for this realm
	chain *temporalchain.Chain
)

func init() {
	chain = temporalchain.NewChain()
}

// SubstrateData simulates the data needed to verify constitutional invariants.
type SubstrateData struct {
	ID            uint64
	HasContradiction bool
	IsLoopClosed  bool
	HasDocumentedGaps bool
	PhiCRatio     float64
	SealHash      string
	Architect     string
	Timestamp     int64
}

// CheckGhostContract verifies that a substrate has no internal contradictions.
func CheckGhostContract(data *SubstrateData) bool {
	return !data.HasContradiction
}

// CheckLoopsealContract ensures the hereditary chain of a substrate is closed and valid.
func CheckLoopsealContract(data *SubstrateData) bool {
	return data.IsLoopClosed
}

// CheckGapContract requires that gaps are documented before canonization.
func CheckGapContract(data *SubstrateData) bool {
	return data.HasDocumentedGaps
}

// CheckPhiContract verifies if the golden ratio (PhiC) is maintained.
// We accept any PhiC > 0.618 as valid for this simulation.
func CheckPhiContract(data *SubstrateData) bool {
	return data.PhiCRatio >= 0.618
}

// VerifyAndAnchor runs all invariant checks and, if successful, anchors the seal to the TemporalChain.
func VerifyAndAnchor(data *SubstrateData) string {
	if !CheckGhostContract(data) {
		return "REJECTED: Ghost Contract violated (internal contradiction detected)."
	}
	if !CheckLoopsealContract(data) {
		return "REJECTED: Loopseal Contract violated (hereditary chain not closed)."
	}
	if !CheckGapContract(data) {
		return "REJECTED: Gap Contract violated (undocumented gaps)."
	}
	if !CheckPhiContract(data) {
		return "REJECTED: Phi Contract violated (PhiC below threshold)."
	}

	// All checks passed, anchor the seal
	seal := &temporalchain.MsgAnchorSeal{
		SubstrateID: data.ID,
		PhiC:        data.PhiCRatio,
		SealHash:    data.SealHash,
		Architect:   data.Architect,
		Timestamp:   data.Timestamp,
	}

	chain.AnchorSeal(seal)
	return "CANONIZED: Substrate anchored successfully."
}

// Render returns a string representation of all anchored seals in the realm.
func Render(path string) string {
	seals := chain.GetSeals()
	if len(seals) == 0 {
		return "No seals anchored yet."
	}

	var sb strings.Builder
	sb.WriteString("# TemporalChain Seals\n\n")
	for _, seal := range seals {
		// Simulating string formatting since simple concatenation is needed in Gno
		sb.WriteString("## Substrate ")
		// Use integer conversion logic (no fmt in standard gno available directly usually, though we can use strconv if needed, let's keep it simple)
		// Assuming we can just append for now, or just use simpler strings.
		// For simplicity, we will output static info if format is hard without strconv
		sb.WriteString("Seal Info:\n")
		sb.WriteString("- Hash: " + seal.SealHash + "\n")
		sb.WriteString("- Architect: " + seal.Architect + "\n\n")
	}
	return sb.String()
}
