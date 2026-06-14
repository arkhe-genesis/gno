package memory

import (
	"math"
	"sync"
)

type EpisodicMemory struct {
	mu       sync.RWMutex
	vectors  [][]float64
	metadata []map[string]interface{}
	dim      int
	maxSize  int
}

func NewEpisodicMemory(dim, maxSize int) *EpisodicMemory {
	return &EpisodicMemory{
		vectors:  make([][]float64, 0),
		metadata: make([]map[string]interface{}, 0),
		dim:      dim,
		maxSize:  maxSize,
	}
}

func (m *EpisodicMemory) Store(vec []float64, meta map[string]interface{}) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if len(m.vectors) >= m.maxSize {
		m.vectors = m.vectors[1:]
		m.metadata = m.metadata[1:]
	}
	m.vectors = append(m.vectors, vec)
	m.metadata = append(m.metadata, meta)
}

func (m *EpisodicMemory) Recall(query []float64, k int) []map[string]interface{} {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if len(m.vectors) == 0 {
		return nil
	}
	type item struct {
		idx int
		sim float64
	}
	items := make([]item, len(m.vectors))
	for i, vec := range m.vectors {
		sim := cosineSimilarity(query, vec)
		items[i] = item{i, sim}
	}
	// ordena por similaridade decrescente
	for i := 0; i < k && i < len(items); i++ {
		for j := i + 1; j < len(items); j++ {
			if items[j].sim > items[i].sim {
				items[i], items[j] = items[j], items[i]
			}
		}
	}
	res := make([]map[string]interface{}, 0, k)
	for i := 0; i < k && i < len(items); i++ {
		res = append(res, m.metadata[items[i].idx])
	}
	return res
}

func cosineSimilarity(a, b []float64) float64 {
	var dot, normA, normB float64
	for i := 0; i < len(a) && i < len(b); i++ {
		dot += a[i] * b[i]
		normA += a[i] * a[i]
		normB += b[i] * b[i]
	}
	if normA == 0 || normB == 0 {
		return 0
	}
	return dot / (math.Sqrt(normA) * math.Sqrt(normB))
}
