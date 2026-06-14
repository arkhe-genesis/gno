package slowbrain

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type SlowBrain struct {
	endpoint   string
	httpClient *http.Client
	schema     map[string]interface{}
}

func NewSlowBrain(endpoint string) *SlowBrain {
	return &SlowBrain{
		endpoint:   endpoint,
		httpClient: &http.Client{Timeout: 30 * time.Second},
		schema: map[string]interface{}{
			"type": "object",
			"properties": map[string]interface{}{
				"reasoning":      map[string]string{"type": "string"},
				"action_vector":  map[string]interface{}{"type": "array", "items": map[string]string{"type": "number"}},
				"confidence":     map[string]string{"type": "number"},
				"safety_override": map[string]string{"type": "boolean"},
			},
			"required": []string{"reasoning", "action_vector", "confidence"},
		},
	}
}

func (sb *SlowBrain) Reason(ctx context.Context, dilemma string) (map[string]interface{}, error) {
	payload := map[string]interface{}{
		"model": "default",
		"messages": []map[string]string{
			{"role": "system", "content": "You are the Slow Brain of Cathedral AGI. Respond in JSON."},
			{"role": "user", "content": dilemma},
		},
		"response_format": map[string]interface{}{
			"type": "json_schema",
			"json_schema": map[string]interface{}{
				"name":   "decision",
				"schema": sb.schema,
				"strict": true,
			},
		},
		"temperature": 0.1,
		"max_tokens":  500,
	}
	body, _ := json.Marshal(payload)
	req, _ := http.NewRequestWithContext(ctx, "POST", sb.endpoint+"/v1/chat/completions", bytes.NewReader(body))
	req.Header.Set("Content-Type", "application/json")
	resp, err := sb.httpClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	var result map[string]interface{}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, err
	}
	// extrai conteúdo do choices[0].message.content
	if choices, ok := result["choices"].([]interface{}); ok && len(choices) > 0 {
		if choice, ok := choices[0].(map[string]interface{}); ok {
			if msg, ok := choice["message"].(map[string]interface{}); ok {
				if content, ok := msg["content"].(string); ok {
					var decision map[string]interface{}
					if err := json.Unmarshal([]byte(content), &decision); err == nil {
						return decision, nil
					}
				}
			}
		}
	}
	return nil, fmt.Errorf("invalid response")
}
