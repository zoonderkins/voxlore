use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{Manager, State};

use crate::error::AppError;
use crate::security::keystore::KeyStore;
use crate::state::AppState;

static NEXT_HEALTH_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// Save an API key for a provider.
#[tauri::command]
pub fn save_api_key(
    provider: String,
    key: String,
    keystore: State<'_, KeyStore>,
) -> Result<(), AppError> {
    crate::app_log!(
        "[settings] save_api_key provider={} key_len={}",
        provider,
        key.trim().len()
    );
    keystore.save_api_key(&provider, &key)
}

/// Check if an API key exists for a provider.
#[tauri::command]
pub fn has_api_key(
    provider: String,
    keystore: State<'_, KeyStore>,
) -> Result<bool, AppError> {
    crate::app_log!("[settings] has_api_key provider={}", provider);
    keystore.has_api_key(&provider)
}

/// Delete an API key for a provider.
#[tauri::command]
pub fn delete_api_key(
    provider: String,
    keystore: State<'_, KeyStore>,
) -> Result<(), AppError> {
    crate::app_log!("[settings] delete_api_key provider={}", provider);
    keystore.delete_api_key(&provider)
}

/// Sync frontend settings to Rust state.
/// Called by the frontend whenever settings change.
#[tauri::command]
pub fn sync_settings(
    state: State<'_, AppState>,
    widget_position: Option<String>,
    floating_window_enabled: Option<bool>,
    stt_language: Option<String>,
    stt_provider: Option<String>,
    stt_model: Option<String>,
    stt_base_url: Option<String>,
    cloud_timeout_secs: Option<u64>,
    debug_logging_enabled: Option<bool>,
) {
    crate::app_log!(
        "[settings] sync_settings widget_position={:?} stt_language={:?} stt_provider={:?} stt_model={:?}",
        widget_position, stt_language, stt_provider, stt_model
    );
    if let Some(pos) = widget_position {
        *state.widget_position.lock().unwrap() = pos;
    }
    if let Some(enabled) = floating_window_enabled {
        *state.floating_window_enabled.lock().unwrap() = enabled;
    }
    if let Some(lang) = stt_language {
        *state.stt_language.lock().unwrap() = lang;
    }
    if let Some(provider) = stt_provider {
        *state.stt_provider.lock().unwrap() = provider;
    }
    if let Some(model) = stt_model {
        let trimmed = model.trim();
        *state.stt_model.lock().unwrap() = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
    if let Some(base_url) = stt_base_url {
        let trimmed = base_url.trim();
        *state.stt_base_url.lock().unwrap() = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.trim_end_matches('/').to_string())
        };
    }
    if let Some(timeout) = cloud_timeout_secs {
        *state.cloud_timeout_secs.lock().unwrap() = timeout.clamp(5, 180);
    }
    if let Some(enabled) = debug_logging_enabled {
        *state.debug_logging_enabled.lock().unwrap() = enabled;
    }
}

/// UI debug bridge from frontend.
#[tauri::command]
pub fn debug_ui_event(
    event: String,
    payload: String,
    state: State<'_, AppState>,
) {
    if !*state.debug_logging_enabled.lock().unwrap() {
        return;
    }
    crate::app_log!("[ui-debug] {} {}", event, payload);
}

#[tauri::command]
pub fn open_devtools(
    app: tauri::AppHandle,
    window_label: Option<String>,
) -> Result<(), AppError> {
    let label = window_label.unwrap_or_else(|| "main".to_string());
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| AppError::Enhancement(format!("Window not found: {label}")))?;
    window.open_devtools();
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHealth {
    pub ok: bool,
    pub has_key: bool,
    pub latency_ms: Option<u128>,
    pub status: String,
}

#[tauri::command]
pub async fn check_provider_health(
    section: String,
    provider: String,
    model: Option<String>,
    endpoint: Option<String>,
    keystore: State<'_, KeyStore>,
) -> Result<ProviderHealth, AppError> {
    if provider == "vosk" {
        return Ok(ProviderHealth {
            ok: true,
            has_key: true,
            latency_ms: None,
            status: "Local provider ready".into(),
        });
    }

    if provider == "ollama" {
        return check_local_http("http://127.0.0.1:11434/api/tags").await;
    }
    if provider == "lmstudio" {
        return check_local_http("http://127.0.0.1:1234/v1/models").await;
    }

    let key_provider = match provider.as_str() {
        "openai_transcribe" => "openai",
        _ => provider.as_str(),
    };

    let api_key = keystore.get_api_key(key_provider)?;
    if api_key.is_none() {
        return Ok(ProviderHealth {
            ok: false,
            has_key: false,
            latency_ms: None,
            status: format!("Missing API key for {key_provider}"),
        });
    }

    let api_key = api_key.unwrap_or_default();
    let timeout = Duration::from_secs(12);
    let started = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| AppError::Enhancement(format!("Health check client error: {e}")))?;

    let response = if section == "enhancement" {
        check_openai_compatible_chat(&client, &provider, &api_key, model, endpoint).await
    } else {
        check_stt_provider(&client, &provider, &api_key, model, endpoint).await
    };

    let elapsed_ms = started.elapsed().as_millis();
    match response {
        Ok(()) => Ok(ProviderHealth {
            ok: true,
            has_key: true,
            latency_ms: Some(elapsed_ms),
            status: format!("OK ({elapsed_ms} ms)"),
        }),
        Err(msg) => Ok(ProviderHealth {
            ok: false,
            has_key: true,
            latency_ms: Some(elapsed_ms),
            status: msg,
        }),
    }
}

async fn check_local_http(url: &str) -> Result<ProviderHealth, AppError> {
    let timeout = Duration::from_secs(6);
    let started = Instant::now();
    let client = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|e| AppError::Enhancement(format!("Health check client error: {e}")))?;
    let resp = client.get(url).send().await;
    let elapsed_ms = started.elapsed().as_millis();
    match resp {
        Ok(r) if r.status().is_success() => Ok(ProviderHealth {
            ok: true,
            has_key: true,
            latency_ms: Some(elapsed_ms),
            status: format!("Local service ready ({elapsed_ms} ms)"),
        }),
        Ok(r) => Ok(ProviderHealth {
            ok: false,
            has_key: true,
            latency_ms: Some(elapsed_ms),
            status: format!("Local service error: HTTP {}", r.status()),
        }),
        Err(e) => Ok(ProviderHealth {
            ok: false,
            has_key: true,
            latency_ms: Some(elapsed_ms),
            status: format!("Local service unavailable: {e}"),
        }),
    }
}

async fn check_stt_provider(
    client: &reqwest::Client,
    provider: &str,
    api_key: &str,
    model: Option<String>,
    endpoint: Option<String>,
) -> Result<(), String> {
    let local_request_id = NEXT_HEALTH_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();

    let endpoint = endpoint
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty());

    if let Some(base_url) = endpoint {
        let model = normalize_compat_model(provider, &base_url, model, "gemini-3-flash");
        let body = serde_json::json!({
          "model": model,
          "messages": [{"role":"user","content":"ping"}],
          "max_tokens": 1,
          "temperature": 0
        });
        let r = client
            .post(format!("{base_url}/chat/completions"))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("OpenAI-compatible STT endpoint network error: {e}"))?;
        let status = r.status();
        let latency_ms = started.elapsed().as_millis();
        let upstream_request_id = get_response_request_id(r.headers());
        crate::app_log!(
            "[healthcheck] section=voice request_id={} provider={} status={} latency_ms={} upstream_request_id={} endpoint_mode=custom",
            local_request_id, provider, status, latency_ms, upstream_request_id
        );
        if status.is_success() {
            return Ok(());
        }
        return Err(format!(
            "OpenAI-compatible STT endpoint API error: HTTP {}",
            status
        ));
    }

    match provider {
        "custom_openai_compatible" => Err(
            "Custom OpenAI-compatible provider requires endpoint configuration".to_string(),
        ),
        "openai" | "openai_transcribe" => {
            let r = client
                .get("https://api.openai.com/v1/models")
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(|e| format!("OpenAI network error: {e}"))?;
            let status = r.status();
            let latency_ms = started.elapsed().as_millis();
            let upstream_request_id = get_response_request_id(r.headers());
            crate::app_log!(
                "[healthcheck] section=voice request_id={} provider={} status={} latency_ms={} upstream_request_id={} endpoint_mode=default",
                local_request_id, provider, status, latency_ms, upstream_request_id
            );
            if status.is_success() {
                Ok(())
            } else {
                Err(format!("OpenAI API error: HTTP {}", status))
            }
        }
        "openrouter" => {
            let r = client
                .get("https://openrouter.ai/api/v1/models")
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(|e| format!("OpenRouter network error: {e}"))?;
            let status = r.status();
            let latency_ms = started.elapsed().as_millis();
            let upstream_request_id = get_response_request_id(r.headers());
            crate::app_log!(
                "[healthcheck] section=voice request_id={} provider={} status={} latency_ms={} upstream_request_id={} endpoint_mode=default",
                local_request_id, provider, status, latency_ms, upstream_request_id
            );
            if status.is_success() {
                Ok(())
            } else {
                Err(format!("OpenRouter API error: HTTP {}", status))
            }
        }
        "elevenlabs" => {
            let r = client
                .get("https://api.elevenlabs.io/v1/user")
                .header("xi-api-key", api_key)
                .send()
                .await
                .map_err(|e| format!("ElevenLabs network error: {e}"))?;
            let status = r.status();
            let latency_ms = started.elapsed().as_millis();
            let upstream_request_id = get_response_request_id(r.headers());
            crate::app_log!(
                "[healthcheck] section=voice request_id={} provider={} status={} latency_ms={} upstream_request_id={} endpoint_mode=default",
                local_request_id, provider, status, latency_ms, upstream_request_id
            );
            if status.is_success() {
                Ok(())
            } else {
                Err(format!("ElevenLabs API error: HTTP {}", status))
            }
        }
        "mistral" => {
            let r = client
                .get("https://api.mistral.ai/v1/models")
                .bearer_auth(api_key)
                .send()
                .await
                .map_err(|e| format!("Mistral network error: {e}"))?;
            let status = r.status();
            let latency_ms = started.elapsed().as_millis();
            let upstream_request_id = get_response_request_id(r.headers());
            crate::app_log!(
                "[healthcheck] section=voice request_id={} provider={} status={} latency_ms={} upstream_request_id={} endpoint_mode=default",
                local_request_id, provider, status, latency_ms, upstream_request_id
            );
            if status.is_success() {
                Ok(())
            } else {
                Err(format!("Mistral API error: HTTP {}", status))
            }
        }
        _ => Err(format!("Unsupported STT provider: {provider}")),
    }
}

async fn check_openai_compatible_chat(
    client: &reqwest::Client,
    provider: &str,
    api_key: &str,
    model: Option<String>,
    endpoint: Option<String>,
) -> Result<(), String> {
    let local_request_id = NEXT_HEALTH_REQUEST_ID.fetch_add(1, Ordering::Relaxed);
    let started = Instant::now();

    let endpoint = endpoint
        .map(|v| v.trim().trim_end_matches('/').to_string())
        .filter(|v| !v.is_empty());

    let (base_url, default_model) = if let Some(custom) = endpoint.as_deref() {
        (custom, "gemini-3-flash")
    } else {
        match provider {
        "custom_openai_compatible" => {
            return Err("Custom OpenAI-compatible provider requires endpoint configuration".to_string());
        }
        "openrouter" => ("https://openrouter.ai/api/v1", "google/gemini-3-flash-preview"),
        "together" => ("https://api.together.xyz/v1", "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo"),
        "groq" => ("https://api.groq.com/openai/v1", "llama-3.1-8b-instant"),
        "openai" => ("https://api.openai.com/v1", "gpt-4o-mini"),
        _ => return Err(format!("Unsupported enhancement provider: {provider}")),
        }
    };
    let model = normalize_compat_model(provider, base_url, model, default_model);

    let body = serde_json::json!({
      "model": model,
      "messages": [{"role":"user","content":"ping"}],
      "max_tokens": 1,
      "temperature": 0
    });

    let r = client
        .post(format!("{base_url}/chat/completions"))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("{provider} network error: {e}"))?;
    let status = r.status();
    let latency_ms = started.elapsed().as_millis();
    let upstream_request_id = get_response_request_id(r.headers());
    let endpoint_mode = if endpoint.is_some() { "custom" } else { "default" };
    crate::app_log!(
        "[healthcheck] section=enhancement request_id={} provider={} status={} latency_ms={} upstream_request_id={} endpoint_mode={}",
        local_request_id, provider, status, latency_ms, upstream_request_id, endpoint_mode
    );
    if status.is_success() {
        Ok(())
    } else {
        Err(format!("{provider} API error: HTTP {}", status))
    }
}

fn get_response_request_id(headers: &reqwest::header::HeaderMap) -> String {
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

fn normalize_compat_model(
    provider: &str,
    base_url: &str,
    model: Option<String>,
    default_model: &str,
) -> String {
    let raw = model
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| default_model.to_string());

    // Accept LiteLLM/Gemini shorthand, and map to OpenRouter id when using OpenRouter endpoint.
    if raw == "gemini-3-flash" || raw == "gemini-3-flash-preview" {
        if provider == "openrouter" && base_url.contains("openrouter.ai") {
            "google/gemini-3-flash-preview".to_string()
        } else {
            "gemini-3-flash".to_string()
        }
    } else {
        raw
    }
}
