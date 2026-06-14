package fastbrain

import (
	"math"
	"math/rand"
	"time"

	"cathedral-agi/pkg/memory"
	"cathedral-agi/pkg/safety"
)

type FastBrainState struct {
	Action          []float64
	Confidence      float64
	SafetyApproved  bool
	SafetyReason    string
	WorldState      []float64 // embedding 288D (simulado)
	CycleTimeMicros int64
}

type FastBrain struct {
	actionDim   int
	maxForce    float64
	lastAction  []float64
	rssm        *RSSM
	memory      *memory.EpisodicMemory
	safety      *safety.SafetyEngine
}

func NewFastBrain(actionDim int, maxForce float64) *FastBrain {
	return &FastBrain{
		actionDim:  actionDim,
		maxForce:   maxForce,
		lastAction: make([]float64, actionDim),
		rssm:       NewRSSM(256, 32, actionDim),
		memory:     memory.NewEpisodicMemory(288, 10000),
		safety:     safety.NewSafetyEngine(maxForce),
	}
}

func (fb *FastBrain) Cycle(observation []float64, reward float64) FastBrainState {
	start := time.Now()

	// 1. Atualiza RSSM (simula visão + ação anterior)
	worldState := fb.rssm.Step(observation, fb.lastAction, reward)

	// 2. Gera ação a partir do estado latente
	action := make([]float64, fb.actionDim)
	for i := 0; i < fb.actionDim; i++ {
		action[i] = math.Tanh(worldState[i]) // simples
	}

	// 3. Validação de segurança (Z3 emulado)
	safe, reason := fb.safety.Check(action)
	approved := safe
	if !approved {
		action = make([]float64, fb.actionDim) // ação zero
	}
	confidence := math.Abs(action[0]) // proxy

	// 4. Memória episódica (armazena estado latente)
	fb.memory.Store(worldState, map[string]interface{}{
		"action":     action,
		"confidence": confidence,
		"safe":       approved,
	})

	// 5. Atualiza última ação
	copy(fb.lastAction, action)

	state := FastBrainState{
		Action:          action,
		Confidence:      confidence,
		SafetyApproved:  approved,
		SafetyReason:    reason,
		WorldState:      worldState,
		CycleTimeMicros: time.Since(start).Microseconds(),
	}
	return state
}

type RSSM struct {
	deterDim  int
	stochDim  int
	actionDim int
	deter     []float64
	stoch     []float64
	// Pesos aleatórios (em produção seriam treinados)
	Wx [][]float64
	Wh [][]float64
}

func NewRSSM(deterDim, stochDim, actionDim int) *RSSM {
	r := &RSSM{
		deterDim:  deterDim,
		stochDim:  stochDim,
		actionDim: actionDim,
		deter:     make([]float64, deterDim),
		stoch:     make([]float64, stochDim),
	}
	// Inicialização aleatória dos pesos
	// FIX: Wx deve aceitar prevState (deterDim+stochDim) + actionDim, totalizando deterDim+stochDim+actionDim
	r.Wx = randomMatrix(deterDim+stochDim, deterDim+stochDim+actionDim)
	r.Wh = randomMatrix(deterDim, deterDim+stochDim)
	return r
}

func (r *RSSM) Step(obs, action []float64, reward float64) []float64 {
	// Concatena estado anterior (deter+stoch) com ação
	prevState := append(r.deter, r.stoch...)
	input := append(prevState, action...)
	// Camada oculta (simplificada)
	hidden := make([]float64, len(r.Wx))
	for i := range hidden {
		var sum float64
		for j := range input {
			sum += r.Wx[i][j] * input[j]
		}
		hidden[i] = math.Tanh(sum)
	}
	// Atualiza estado determinístico e estocástico
	newDeter := make([]float64, r.deterDim)
	for i := range newDeter {
		var sum float64
		for j := range hidden {
			sum += r.Wh[i][j] * hidden[j]
		}
		newDeter[i] = math.Tanh(sum)
	}
	// Simples: estado estocástico é ruído normalizado
	newStoch := make([]float64, r.stochDim)
	for i := range newStoch {
		newStoch[i] = rand.NormFloat64() * 0.1
	}
	copy(r.deter, newDeter)
	copy(r.stoch, newStoch)
	// Retorna concatenado (288D)
	return append(newDeter, newStoch...)
}

func randomMatrix(rows, cols int) [][]float64 {
	m := make([][]float64, rows)
	for i := range m {
		m[i] = make([]float64, cols)
		for j := range m[i] {
			m[i][j] = rand.Float64()*0.1 - 0.05
		}
	}
	return m
}
