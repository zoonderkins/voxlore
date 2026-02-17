use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use serde_json::json;

use super::{build_enhancement_prompt, EnhancementConfig, EnhancementEngine};
use crate::error::AppError;

/// OpenAI-compatible enhancement engine.
/// Works with OpenRouter, Together, Groq, DeepSeek, and any provider
/// that supports the OpenAI chat completions API format.
pub struct OpenAiCompatEngine {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

impl OpenAiCompatEngine {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Create engine for specific well-known providers.
    pub fn for_provider(api_key: String, provider: &str) -> Self {
        let base_url = match provider {
            "openrouter" => "https://openrouter.ai/api/v1",
            "together" => "https://api.together.xyz/v1",
            "groq" => "https://api.groq.com/openai/v1",
            "deepseek" => "https://api.deepseek.com/v1",
            "openai" => "https://api.openai.com/v1",
            _ => "https://api.openai.com/v1",
        };
        Self::new(api_key, base_url.to_string())
    }

    fn normalize_model(&self, model: &str) -> String {
        let raw = model.trim();
        if raw == "gemini-3-flash" || raw == "gemini-3-flash-preview" {
            if self.base_url.contains("openrouter.ai") {
                "google/gemini-3-flash-preview".to_string()
            } else {
                "gemini-3-flash".to_string()
            }
        } else {
            raw.to_string()
        }
    }

    fn next_request_id() -> u64 {
        NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
    }

    fn response_request_id(headers: &reqwest::header::HeaderMap) -> String {
        const CANDIDATES: [&str; 4] = ["x-request-id", "request-id", "x-correlation-id", "trace-id"];
        for key in CANDIDATES {
            if let Some(value) = headers.get(key).and_then(|v| v.to_str().ok()) {
                if !value.trim().is_empty() {
                    return value.to_string();
                }
            }
        }
        "n/a".to_string()
    }
}

impl EnhancementEngine for OpenAiCompatEngine {
    async fn enhance(&self, text: &str, config: &EnhancementConfig) -> Result<String, AppError> {
        let system_prompt = build_enhancement_prompt(config);

        let body = json!({
            "model": self.normalize_model(&config.model),
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": text }
            ],
            "temperature": 0.3,
            "max_tokens": 2048,
        });

        let started = Instant::now();
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Enhancement(format!("Request failed: {e}")))?;
        let status = response.status();
        let upstream_request_id = Self::response_request_id(response.headers());
        let local_request_id = Self::next_request_id();
        let latency_ms = started.elapsed().as_millis();
        let endpoint_mode = if self.base_url.contains("openrouter.ai")
            || self.base_url.contains("openai.com")
            || self.base_url.contains("together.xyz")
            || self.base_url.contains("groq.com")
        {
            "default"
        } else {
            "custom"
        };
        eprintln!(
            "[enhancement-http] provider=openai_compat request_id={} upstream_request_id={} status={} latency_ms={} endpoint_mode={}",
            local_request_id, upstream_request_id, status, latency_ms, endpoint_mode
        );

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AppError::Enhancement(format!(
                "API error ({status}): {body}"
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Enhancement(format!("Failed to parse response: {e}")))?;

        let enhanced = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or(text)
            .trim()
            .to_string();

        Ok(enhanced)
    }

    fn provider_name(&self) -> &str {
        "OpenAI Compatible"
    }
}
