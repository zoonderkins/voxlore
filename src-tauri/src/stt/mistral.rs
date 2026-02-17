use reqwest::multipart;

use super::{CloudSttEngine, SttConfig, SttResult};
use crate::error::AppError;

/// Mistral Vox STT engine.
pub struct MistralEngine {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl MistralEngine {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            model: model.unwrap_or_else(|| "mistral-vox-latest".to_string()),
            client: reqwest::Client::new(),
        }
    }
}

impl CloudSttEngine for MistralEngine {
    async fn transcribe(&self, audio_data: &[u8], config: &SttConfig) -> Result<SttResult, AppError> {
        let audio_part = multipart::Part::bytes(audio_data.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Stt(format!("Failed to create multipart: {e}")))?;

        let form = multipart::Form::new()
            .part("file", audio_part)
            .text("model", self.model.clone())
            .text("language", config.language.clone());

        let response = self
            .client
            .post("https://api.mistral.ai/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Stt(format!("Mistral request failed: {e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::Stt(format!("Failed to read response: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Stt(format!(
                "Mistral API error ({status}): {body}"
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| AppError::Stt(format!("Failed to parse response: {e}")))?;

        let text = json["text"].as_str().unwrap_or_default().to_string();

        Ok(SttResult {
            text,
            confidence: None,
            language_detected: None,
        })
    }

    fn provider_name(&self) -> &str {
        "Mistral Vox"
    }
}
