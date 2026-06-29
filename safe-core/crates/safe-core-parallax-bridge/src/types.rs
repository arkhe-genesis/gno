use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InferRequest {
    pub model_name: String,
    pub prompt: String,
    pub system_prompt: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub params: SamplingParams,
    pub tools: Vec<ToolDefinition>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct SamplingParams {
    pub temperature: f32,
    pub top_p: f32,
    pub top_k: Option<usize>,
    pub max_tokens: usize,
    pub stop_sequences: Vec<String>,
    pub seed: Option<u64>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            top_p: 0.9,
            top_k: None,
            max_tokens: 2048,
            stop_sequences: Vec::new(),
            seed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct InferResponse {
    pub id: String,
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
    pub finish_reason: String,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct HealthResponse {
    pub ready: bool,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct EmbedRequest {
    pub model_name: String,
    pub texts: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EmbedResponse {
    pub embeddings: Vec<Embedding>,
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub values: Vec<f32>,
}
