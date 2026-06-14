package main

import (
	"context"
	"fmt"
	"log"
	"math/rand"
	"time"

	"cathedral-agi/pkg/fastbrain"
	"cathedral-agi/pkg/router"
	"cathedral-agi/pkg/slowbrain"
)

type CathedralAGI struct {
	fast       *fastbrain.FastBrain
	slow       *slowbrain.SlowBrain
	router     *router.Router
	cycleCount int
}

func NewCathedralAGI(actionDim int, maxForce float64, slowEndpoint string) *CathedralAGI {
	return &CathedralAGI{
		fast:   fastbrain.NewFastBrain(actionDim, maxForce),
		slow:   slowbrain.NewSlowBrain(slowEndpoint),
		router: router.NewRouter(0.3),
	}
}

func (agi *CathedralAGI) Cycle(observation []float64, reward float64) (action []float64, route string, err error) {
	agi.cycleCount++
	fastState := agi.fast.Cycle(observation, reward)

	route = agi.router.Decide(fastState, true)

	if route == "slow" {
		dilemma := fmt.Sprintf("Fast Brain proposed action %v with confidence %.2f, but safety: %s", fastState.Action, fastState.Confidence, fastState.SafetyReason)
		decision, err := agi.slow.Reason(context.Background(), dilemma)
		if err != nil {
			log.Printf("Slow Brain failed: %v, fallback to fast action", err)
			return fastState.Action, "fast-fallback", nil
		}
		if act, ok := decision["action_vector"].([]interface{}); ok {
			action = make([]float64, len(act))
			for i, v := range act {
				action[i] = v.(float64)
			}
			return action, "slow", nil
		}
	}
	return fastState.Action, route, nil
}

func main() {
	rand.Seed(time.Now().UnixNano())
	agi := NewCathedralAGI(4, 10.0, "http://localhost:8000")

	fmt.Println("Cathedral AGI Go – Starting cognitive loop (10 cycles)")
	for i := 0; i < 10; i++ {
		// Simula observação (embedding de câmera/sensores)
		obs := make([]float64, 256)
		for j := range obs {
			obs[j] = rand.Float64()*2 - 1
		}
		reward := 0.0
		if i == 5 {
			reward = 1.0 // simula acerto
		}
		action, route, err := agi.Cycle(obs, reward)
		if err != nil {
			log.Fatal(err)
		}
		fmt.Printf("Cycle %2d | route: %-12s | action: %v\n", i+1, route, action)
	}
}
