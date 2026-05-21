package temporalchain

// MsgAnchorSeal represents a transaction to anchor a canonized substrate seal.
type MsgAnchorSeal struct {
	SubstrateID uint64
	PhiC        float64
	SealHash    string
	Architect   string // ORCID or identity
	Timestamp   int64  // Unix timestamp
}

// Chain is the in-memory representation of the TemporalChain for this package/realm.
type Chain struct {
	seals []*MsgAnchorSeal
}

// NewChain creates a new TemporalChain.
func NewChain() *Chain {
	return &Chain{
		seals: make([]*MsgAnchorSeal, 0),
	}
}

// AnchorSeal adds a new canonized seal to the TemporalChain.
func (c *Chain) AnchorSeal(msg *MsgAnchorSeal) {
	c.seals = append(c.seals, msg)
}

// GetSeals returns all anchored seals in the TemporalChain.
func (c *Chain) GetSeals() []*MsgAnchorSeal {
	return c.seals
}
