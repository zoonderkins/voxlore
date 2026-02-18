use tauri::State;

use crate::error::AppError;
use crate::security::keystore::KeyStore;
use crate::stt::elevenlabs::ElevenLabsEngine;
use crate::stt::mistral::MistralEngine;
use crate::stt::openai_whisper::OpenAiWhisperEngine;
use crate::stt::openrouter_audio::OpenRouterAudioEngine;
use crate::stt::vosk_engine::VoskManager;
use crate::stt::converter;
use crate::stt::{CloudSttEngine, SttConfig, SttProvider, SttResult};

/// Transcribe audio data using the specified STT provider.
///
/// For cloud providers, `audio_data` is WAV-encoded audio.
/// For Vosk, `audio_data` is raw PCM i16 LE samples at 16kHz.
#[tauri::command]
pub async fn transcribe_audio(
    audio_data: Vec<u8>,
    provider: SttProvider,
    language: String,
    model: Option<String>,
    keystore: State<'_, KeyStore>,
    vosk: State<'_, VoskManager>,
) -> Result<SttResult, AppError> {
    crate::app_log!(
        "[stt] transcribe_audio provider={:?} language={} model={:?}",
        provider, language, model
    );
    let needs_s2t = converter::needs_s2t_conversion(&language);

    let config = SttConfig {
        language,
        sample_rate: 16000,
    };

    let mut result = match provider {
        SttProvider::Vosk => {
            vosk.transcribe(&audio_data, config.sample_rate as f32)
        }
        SttProvider::ElevenLabs => {
            let api_key = get_api_key(&keystore, "elevenlabs")?;
            let engine = ElevenLabsEngine::new(api_key, model);
            engine.transcribe(&audio_data, &config).await
        }
        SttProvider::OpenAI => {
            let api_key = get_api_key(&keystore, "openai")?;
            let engine = OpenAiWhisperEngine::new(api_key, model, None);
            engine.transcribe(&audio_data, &config).await
        }
        SttProvider::OpenAITranscribe => {
            let api_key = get_api_key(&keystore, "openai")?;
            let transcribe_model = model.or_else(|| Some("gpt-4o-mini-transcribe".to_string()));
            let engine = OpenAiWhisperEngine::new(api_key, transcribe_model, None);
            engine.transcribe(&audio_data, &config).await
        }
        SttProvider::OpenRouter => {
            let api_key = get_api_key(&keystore, "openrouter")?;
            let engine = OpenRouterAudioEngine::new(api_key, model, None);
            engine.transcribe(&audio_data, &config).await
        }
        SttProvider::CustomOpenAiCompatible => {
            let api_key = get_api_key(&keystore, "custom_openai_compatible")?;
            Err(AppError::Stt(format!(
                "Custom OpenAI-compatible STT requires endpoint in current recording pipeline. Provider key exists: {}",
                !api_key.is_empty()
            )))
        }
        SttProvider::Mistral => {
            let api_key = get_api_key(&keystore, "mistral")?;
            let engine = MistralEngine::new(api_key, model);
            engine.transcribe(&audio_data, &config).await
        }
    }?;

    // Convert Simplified → Traditional Chinese for zh-TW users
    if needs_s2t {
        result.text = converter::simplified_to_traditional(&result.text);
    }

    Ok(result)
}

fn get_api_key(keystore: &KeyStore, provider: &str) -> Result<String, AppError> {
    keystore
        .get_api_key(provider)?
        .ok_or_else(|| AppError::Stt(format!("No API key configured for {provider}")))
}
