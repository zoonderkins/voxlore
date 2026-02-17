pub mod converter;
pub mod elevenlabs;
pub mod mistral;
pub mod openai_whisper;
pub mod openrouter_audio;
pub mod vosk_engine;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Configuration for STT sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    pub language: String,
    pub sample_rate: u32,
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            sample_rate: 16000,
        }
    }
}

/// Result from STT processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttResult {
    pub text: String,
    pub confidence: Option<f32>,
    pub language_detected: Option<String>,
}

/// Trait for cloud STT engines that process complete audio buffers.
#[allow(async_fn_in_trait)]
pub trait CloudSttEngine: Send + Sync {
    /// Transcribe a complete audio buffer (WAV format).
    async fn transcribe(&self, audio_data: &[u8], config: &SttConfig) -> Result<SttResult, AppError>;

    /// Get the provider name for display.
    #[allow(dead_code)]
    fn provider_name(&self) -> &str;
}

/// Supported STT providers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SttProvider {
    #[serde(rename = "vosk")]
    Vosk,
    #[serde(rename = "elevenlabs")]
    ElevenLabs,
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "openai_transcribe")]
    OpenAITranscribe,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "custom_openai_compatible")]
    CustomOpenAiCompatible,
    #[serde(rename = "mistral")]
    Mistral,
}
