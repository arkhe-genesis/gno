package safety

import (
	"fmt"
	"math"
)

type SafetyEngine struct {
	maxForce float64
}

func NewSafetyEngine(maxForce float64) *SafetyEngine {
	return &SafetyEngine{maxForce: maxForce}
}

func (s *SafetyEngine) Check(action []float64) (bool, string) {
	force := 0.0
	for _, a := range action {
		force += a * a
	}
	force = math.Sqrt(force)
	if force > s.maxForce {
		return false, fmt.Sprintf("força %.2fN excede limite %.2fN", force, s.maxForce)
	}
	// Regra adicional: se ação tiver componente > 0.8, é arriscado
	for _, a := range action {
		if math.Abs(a) > 0.8 {
			return false, "ação muito agressiva"
		}
	}
	return true, "seguro"
}
