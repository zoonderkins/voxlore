use serde_json::json;

use super::{build_enhancement_prompt, EnhancementConfig, EnhancementEngine};
use crate::error::AppError;

/// Ollama / LM Studio local enhancement engine.
pub struct OllamaEngine {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaEngine {
    pub fn new(base_url: Option<String>) -> Self {
        Self {
            base_url: base_url
                .unwrap_or_else(|| "http://localhost:11434".to_string())
                .trim_end_matches('/')
                .to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create for LM Studio (default port 1234).
    pub fn lm_studio() -> Self {
        Self::new(Some("http://localhost:1234/v1".to_string()))
    }
}

impl EnhancementEngine for OllamaEngine {
    async fn enhance(&self, text: &str, config: &EnhancementConfig) -> Result<String, AppError> {
        let system_prompt = build_enhancement_prompt(config);

        // Ollama uses /api/chat endpoint
        let body = json!({
            "model": config.model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": text }
            ],
            "stream": false,
        });

        let url = if self.base_url.contains("/v1") {
            // LM Studio uses OpenAI-compatible format
            format!("{}/chat/completions", self.base_url)
        } else {
            // Ollama native format
            format!("{}/api/chat", self.base_url)
        };

        let response = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Enhancement(format!("Local LLM request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Enhancement(format!(
                "Local LLM error ({status}): {body}"
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Enhancement(format!("Failed to parse response: {e}")))?;

        // Handle both Ollama and OpenAI-compatible response formats
        let enhanced = json["message"]["content"]
            .as_str()
            .or_else(|| json["choices"][0]["message"]["content"].as_str())
            .unwrap_or(text)
            .trim()
            .to_string();

        Ok(enhanced)
    }

    fn provider_name(&self) -> &str {
        "Ollama"
    }
}
