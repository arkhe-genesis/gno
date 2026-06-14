package router

import (
	"cathedral-agi/pkg/fastbrain"
)

type Router struct {
	confidenceThreshold float64
}

func NewRouter(threshold float64) *Router {
	return &Router{confidenceThreshold: threshold}
}

func (r *Router) Decide(fastState fastbrain.FastBrainState, slowAvailable bool) string {
	if !fastState.SafetyApproved && slowAvailable {
		return "slow"
	}
	if fastState.Confidence < r.confidenceThreshold && slowAvailable {
		return "slow"
	}
	return "fast"
}
