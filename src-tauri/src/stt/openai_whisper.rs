use reqwest::multipart;

use super::{CloudSttEngine, SttConfig, SttResult};
use crate::error::AppError;

/// OpenAI Whisper STT engine.
pub struct OpenAiWhisperEngine {
    api_key: String,
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAiWhisperEngine {
    pub fn new(api_key: String, model: Option<String>, base_url: Option<String>) -> Self {
        let base_url = base_url
            .map(|v| v.trim().trim_end_matches('/').to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        let model = model.unwrap_or_else(|| "whisper-1".to_string());
        Self {
            api_key,
            model,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    fn build_prompt(language: &str) -> Option<String> {
        let lang = language.to_ascii_lowercase();
        if lang == "zh" || lang == "zh-tw" {
            Some("請輸出臺灣繁體中文逐字稿，只回傳辨識文字。".to_string())
        } else if lang.starts_with("ja") {
            Some("日本語で文字起こしし、認識結果のみ返してください。".to_string())
        } else if lang.starts_with("en") {
            Some("Transcribe in English and return transcript text only.".to_string())
        } else {
            None
        }
    }
}

impl CloudSttEngine for OpenAiWhisperEngine {
    async fn transcribe(&self, audio_data: &[u8], config: &SttConfig) -> Result<SttResult, AppError> {
        let audio_part = multipart::Part::bytes(audio_data.to_vec())
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| AppError::Stt(format!("Failed to create multipart: {e}")))?;

        let mut form = multipart::Form::new()
            .part("file", audio_part)
            .text("model", self.model.clone())
            .text("language", config.language.clone())
            .text("response_format", "json".to_string());
        if let Some(prompt) = Self::build_prompt(&config.language) {
            form = form.text("prompt", prompt);
        }

        let response = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Stt(format!("OpenAI request failed: {e}")))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| AppError::Stt(format!("Failed to read response: {e}")))?;

        if !status.is_success() {
            return Err(AppError::Stt(format!("OpenAI API error ({status}): {body}")));
        }

        let json: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| AppError::Stt(format!("Failed to parse response: {e}")))?;

        let text = json["text"].as_str().unwrap_or_default().to_string();

        Ok(SttResult {
            text,
            confidence: None,
            language_detected: json["language"].as_str().map(String::from),
        })
    }

    fn provider_name(&self) -> &str {
        "OpenAI Whisper"
    }
}
