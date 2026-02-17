use tauri::State;

use crate::enhancement::openai_compat::OpenAiCompatEngine;
use crate::enhancement::ollama::OllamaEngine;
use crate::enhancement::{EnhancementConfig, EnhancementEngine, EnhancementMode};
use crate::error::AppError;
use crate::security::keystore::KeyStore;

fn has_mixed_script(input: &str) -> bool {
    let has_cjk = input.chars().any(|ch| {
        ('\u{4E00}'..='\u{9FFF}').contains(&ch)
            || ('\u{3400}'..='\u{4DBF}').contains(&ch)
            || ('\u{F900}'..='\u{FAFF}').contains(&ch)
    });
    let has_latin = input.chars().any(|ch| ch.is_ascii_alphabetic());
    has_cjk && has_latin
}

/// Enhance text using the specified LLM provider.
#[tauri::command]
pub async fn enhance_text(
    text: String,
    provider: String,
    model: String,
    language: Option<String>,
    endpoint: Option<String>,
    keystore: State<'_, KeyStore>,
) -> Result<String, AppError> {
    let is_local = provider == "ollama" || provider == "lmstudio";
    eprintln!(
        "[enhancement] request provider={} model={} language={} is_local={}",
        provider,
        model,
        language.clone().unwrap_or_else(|| "en".to_string()),
        is_local
    );

    let config = EnhancementConfig {
        mode: EnhancementMode::FixGrammar,
        language: language.unwrap_or_else(|| "en".to_string()),
        model,
        custom_prompt: None,
        source_has_mixed_script: has_mixed_script(&text),
    };

    match provider.as_str() {
        "ollama" => {
            let engine = OllamaEngine::new(None);
            engine.enhance(&text, &config).await
        }
        "lmstudio" => {
            let engine = OllamaEngine::lm_studio();
            engine.enhance(&text, &config).await
        }
        _ => {
            if provider == "custom_openai_compatible"
                && endpoint
                    .as_ref()
                    .map(|v| v.trim().is_empty())
                    .unwrap_or(true)
            {
                return Err(AppError::Enhancement(
                    "Custom OpenAI-compatible provider requires endpoint.".to_string(),
                ));
            }
            let maybe_api_key = keystore.get_api_key(&provider)?;
            eprintln!(
                "[enhancement] cloud provider key_exists={}",
                maybe_api_key.is_some()
            );
            let api_key = maybe_api_key
                .ok_or_else(|| AppError::Enhancement(format!("No API key configured for {provider}")))?;
            let engine = if let Some(custom_endpoint) = endpoint
                .map(|v| v.trim().trim_end_matches('/').to_string())
                .filter(|v| !v.is_empty())
            {
                OpenAiCompatEngine::new(api_key, custom_endpoint)
            } else {
                OpenAiCompatEngine::for_provider(api_key, &provider)
            };
            engine.enhance(&text, &config).await
        }
    }
}
