use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use base64::Engine as _;
use serde_json::json;

use super::{CloudSttEngine, SttConfig, SttResult};
use crate::error::AppError;

/// OpenRouter audio STT engine (experimental).
/// Uses OpenAI-compatible chat/completions with `input_audio`.
pub struct OpenRouterAudioEngine {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

impl OpenRouterAudioEngine {
    pub fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let base_url = base_url
            .map(|v| v.trim().trim_end_matches('/').to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string());
        let model = match model
            .unwrap_or_else(|| "google/gemini-3-flash-preview".to_string())
            .trim()
        {
            "gemini-3-flash" | "gemini-3-flash-preview" if base_url.contains("openrouter.ai") => {
                "google/gemini-3-flash-preview".to_string()
            }
            other => other.to_string(),
        };
        Self {
            api_key,
            model,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    fn transcription_prompt(language: &str) -> String {
        if language.eq_ignore_ascii_case("zh") || language.eq_ignore_ascii_case("zh-tw") {
            "請直接輸出「臺灣繁體中文」逐字稿，不要解釋，不要額外標點修飾。".to_string()
        } else {
            "Return plain transcript text only. No explanation.".to_string()
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

impl CloudSttEngine for OpenRouterAudioEngine {
    async fn transcribe(&self, audio_data: &[u8], config: &SttConfig) -> Result<SttResult, AppError> {
        let b64_audio = base64::engine::general_purpose::STANDARD.encode(audio_data);
        let prompt = Self::transcription_prompt(&config.language);

        let body = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": [
                    { "type": "text", "text": prompt },
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": b64_audio,
                            "format": "wav"
                        }
                    }
                ]
            }],
            "temperature": 0.0,
            "max_tokens": 4096
        });

        let mut request = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body);
        if self.base_url.contains("openrouter.ai") {
            request = request
                .header("HTTP-Referer", "https://voxlore.app")
                .header("X-Title", "Voxlore");
        }
        let started = Instant::now();
        let response = request
            .send()
            .await
            .map_err(|e| AppError::Stt(format!("OpenRouter request failed: {e}")))?;

        let status = response.status();
        let request_id = Self::response_request_id(response.headers());
        let body_text = response
            .text()
            .await
            .map_err(|e| AppError::Stt(format!("Failed to read response: {e}")))?;
        let latency_ms = started.elapsed().as_millis();
        let local_request_id = Self::next_request_id();
        eprintln!(
            "[stt-http] provider=openrouter request_id={} upstream_request_id={} status={} latency_ms={}",
            local_request_id, request_id, status, latency_ms
        );

        if !status.is_success() {
            return Err(AppError::Stt(format!(
                "OpenRouter API error ({status}): {body_text}"
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&body_text)
            .map_err(|e| AppError::Stt(format!("Failed to parse OpenRouter response: {e}")))?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .trim()
            .to_string();

        Ok(SttResult {
            text,
            confidence: None,
            language_detected: Some(config.language.clone()),
        })
    }

    fn provider_name(&self) -> &str {
        "OpenRouter Audio (Experimental)"
    }
}
