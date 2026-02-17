use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Local;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use crate::audio::capture::AudioCapture;
use crate::audio::wav;
use crate::error::AppError;
use crate::security::keystore::KeyStore;
use crate::state::AppState;
use crate::stt::converter;
use crate::stt::elevenlabs::ElevenLabsEngine;
use crate::stt::mistral::MistralEngine;
use crate::stt::openai_whisper::OpenAiWhisperEngine;
use crate::stt::openrouter_audio::OpenRouterAudioEngine;
use crate::stt::{CloudSttEngine, SttConfig, SttProvider};
use crate::stt::vosk_engine::VoskManager;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordingResult {
    pub text: String,
    pub audio_path: Option<String>,
    pub text_path: Option<String>,
    pub duration_secs: f32,
}

const SAMPLE_RATE: u32 = 16000;

/// Start recording from the default microphone.
///
/// AudioCapture + cpal::Stream live entirely inside a `spawn_blocking` task
/// because cpal::Stream is `!Send`. We control it via an `AtomicBool` stop signal.
#[tauri::command]
pub async fn start_recording(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    // Check microphone permission before attempting to record
    let mic_status = tokio::task::spawn_blocking(super::permissions::microphone_status)
        .await
        .map_err(|e| AppError::Audio(format!("Permission check failed: {e}")))?;

    match mic_status.as_str() {
        "granted" => {
            eprintln!("[recording] Microphone permission: granted");
        }
        "not_determined" => {
            eprintln!("[recording] Microphone permission: not_determined, requesting...");
            let granted = tokio::task::spawn_blocking(super::permissions::request_microphone_access)
                .await
                .map_err(|e| AppError::Audio(format!("Permission request failed: {e}")))?;
            if !granted {
                let _ = app.emit(
                    "recording:status",
                    serde_json::json!({"status": "error", "message": "Microphone access denied. Please grant permission in System Settings > Privacy > Microphone."}),
                );
                return Err(AppError::Audio("Microphone access denied".into()));
            }
            eprintln!("[recording] Microphone permission: granted after request");
        }
        status => {
            eprintln!("[recording] Microphone permission: {status}");
            let _ = app.emit(
                "recording:status",
                serde_json::json!({"status": "error", "message": "Microphone access denied. Please grant permission in System Settings > Privacy > Microphone."}),
            );
            return Err(AppError::Audio(format!("Microphone permission {status}. Open System Settings > Privacy > Microphone.")));
        }
    }

    // Check if already recording
    {
        let task = state.recording_task.lock().unwrap();
        if task.is_some() {
            return Err(AppError::Audio("Already recording".into()));
        }
    }

    let stop = Arc::new(AtomicBool::new(false));
    // Signal that start_recording has begun (ready flag for stop to wait on)
    let ready = Arc::new(AtomicBool::new(false));
    *state.stop_signal.lock().unwrap() = Some(stop.clone());

    let app_handle = app.clone();
    let ready_clone = ready.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let mut capture = AudioCapture::new();
        if let Err(e) = capture.start() {
            let msg = format!("Audio capture failed: {e}");
            eprintln!("{msg}");
            let _ = app_handle.emit(
                "recording:status",
                serde_json::json!({"status": "error", "message": msg}),
            );
            ready_clone.store(true, Ordering::Release);
            return Vec::new();
        }

        let receiver = match capture.take_receiver() {
            Some(rx) => rx,
            None => {
                let msg = "No audio receiver available";
                eprintln!("{msg}");
                let _ = app_handle.emit(
                    "recording:status",
                    serde_json::json!({"status": "error", "message": msg}),
                );
                ready_clone.store(true, Ordering::Release);
                return Vec::new();
            }
        };

        // Signal that recording has started successfully
        ready_clone.store(true, Ordering::Release);

        let mut buffer: Vec<i16> = Vec::new();
        let mut last_emit = Instant::now();

        loop {
            // Check stop signal BEFORE waiting — critical for quick stop
            if stop.load(Ordering::Relaxed) {
                eprintln!("[recording] Stop signal detected, breaking loop");
                break;
            }

            match receiver.recv_timeout(Duration::from_millis(50)) {
                Ok(chunk) => {
                    if last_emit.elapsed().as_millis() >= 33 {
                        let rms = wav::calculate_rms(&chunk);
                        let _ = app_handle.emit(
                            "recording:audio-level",
                            serde_json::json!({"level": rms}),
                        );
                        last_emit = Instant::now();
                    }
                    buffer.extend(chunk);
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }

        capture.stop();
        eprintln!("[recording] Collected {} samples ({:.1}s)", buffer.len(), buffer.len() as f32 / SAMPLE_RATE as f32);
        buffer
    });

    // Wait briefly for the blocking task to signal readiness
    // This prevents stop_recording from racing ahead before audio starts
    let ready_wait = ready.clone();
    tokio::task::spawn_blocking(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !ready_wait.load(Ordering::Acquire) && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
    })
    .await
    .ok();

    *state.recording_task.lock().unwrap() = Some(handle);

    let _ = app.emit(
        "recording:status",
        serde_json::json!({"status": "recording"}),
    );

    eprintln!("[recording] Started");
    Ok(())
}

/// Stop recording, save WAV + transcription, return result.
///
/// This function handles the race condition where `stop_recording` may be called
/// before `start_recording` has finished storing its state. It will wait up to
/// 5 seconds for the recording to become available before giving up.
#[tauri::command]
pub async fn stop_recording(
    app: AppHandle,
    output_dir: Option<String>,
    state: State<'_, AppState>,
    vosk: State<'_, VoskManager>,
    keystore: State<'_, KeyStore>,
) -> Result<RecordingResult, AppError> {
    eprintln!("[recording] Stopping...");

    let deadline = Instant::now() + Duration::from_secs(5);

    // Wait for start_recording to store the stop signal, then take it
    loop {
        {
            let signal = state.stop_signal.lock().unwrap().take();
            if let Some(stop) = signal {
                stop.store(true, Ordering::Relaxed);
                eprintln!("[recording] Stop signal sent");
                break;
            }
        }
        if Instant::now() > deadline {
            eprintln!("[recording] Timed out waiting for stop signal");
            return Err(AppError::Audio("No recording in progress".into()));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Wait for start_recording to store the task handle
    let handle = loop {
        {
            let task = state.recording_task.lock().unwrap().take();
            if let Some(h) = task {
                eprintln!("[recording] Got recording task handle");
                break h;
            }
        }
        if Instant::now() > deadline {
            eprintln!("[recording] Timed out waiting for recording task");
            return Err(AppError::Audio("Recording task not ready".into()));
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    };

    let buffer = handle
        .await
        .map_err(|e| AppError::Audio(format!("Recording task failed: {e}")))?;

    eprintln!("[recording] Buffer size: {} samples", buffer.len());

    if buffer.is_empty() {
        let _ = app.emit(
            "recording:status",
            serde_json::json!({"status": "error", "message": "No audio captured. Check microphone permissions."}),
        );
        return Ok(RecordingResult {
            text: String::new(),
            audio_path: None,
            text_path: None,
            duration_secs: 0.0,
        });
    }

    let duration_secs = buffer.len() as f32 / SAMPLE_RATE as f32;

    // Determine output directory
    let dir = resolve_output_dir(output_dir)?;
    fs::create_dir_all(&dir)?;

    // Generate timestamped filename
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let base_name = format!("recording_{timestamp}");

    // Save WAV
    let wav_path = dir.join(format!("{base_name}.wav"));
    let wav_data = wav::encode_wav(&buffer, SAMPLE_RATE);
    fs::write(&wav_path, &wav_data)?;
    eprintln!("[recording] Saved WAV: {} ({} bytes)", wav_path.display(), wav_data.len());

    let provider = state.stt_provider.lock().unwrap().clone();
    let language = state.stt_language.lock().unwrap().clone();
    let model = state.stt_model.lock().unwrap().clone();
    let stt_base_url = state.stt_base_url.lock().unwrap().clone();
    let cloud_timeout_secs = *state.cloud_timeout_secs.lock().unwrap();
    eprintln!(
        "[recording] STT settings provider={} language={} model={:?}",
        provider, language, model
    );

    let processing_message = if provider == "vosk" {
        "Processing transcription locally...".to_string()
    } else {
        format!(
            "Processing via cloud AI ({provider})... If network is slow, this may timeout."
        )
    };
    let _ = app.emit(
        "recording:status",
        serde_json::json!({"status": "processing", "message": processing_message}),
    );

    let text = match transcribe_with_selected_provider(
        &app,
        &buffer,
        &provider,
        &language,
        model,
        stt_base_url,
        cloud_timeout_secs,
        &vosk,
        &keystore,
    )
    .await
    {
        Ok(text) => text,
        Err(e) => {
            eprintln!("[recording] Transcription failed: {e}");
            let _ = app.emit(
                "recording:status",
                serde_json::json!({"status": "error", "message": format!("Transcription failed: {e}")}),
            );
            String::new()
        }
    };

    // Save transcription text
    let txt_path = dir.join(format!("{base_name}.txt"));
    fs::write(&txt_path, &text)?;
    eprintln!("[recording] Saved TXT: {}", txt_path.display());

    let _ = app.emit("recording:status", serde_json::json!({"status": "done"}));

    Ok(RecordingResult {
        text,
        audio_path: Some(wav_path.display().to_string()),
        text_path: Some(txt_path.display().to_string()),
        duration_secs,
    })
}

async fn transcribe_with_selected_provider(
    app: &AppHandle,
    samples: &[i16],
    provider_raw: &str,
    language: &str,
    model: Option<String>,
    base_url: Option<String>,
    cloud_timeout_secs: u64,
    vosk: &VoskManager,
    keystore: &KeyStore,
) -> Result<String, AppError> {
    let timeout_secs = cloud_timeout_secs.max(5).min(180);

    let provider: SttProvider = serde_json::from_str(&format!("\"{}\"", provider_raw))
        .map_err(|_| AppError::Stt(format!("Unsupported STT provider: {provider_raw}")))?;

    let needs_s2t = converter::needs_s2t_conversion(language);
    let config = SttConfig {
        language: language.to_string(),
        sample_rate: SAMPLE_RATE,
    };

    let mut text = match provider {
        SttProvider::Vosk => {
            if !vosk.is_loaded() {
                let _ = app.emit(
                    "recording:status",
                    serde_json::json!({"status": "error", "message": "Vosk model not loaded. Please download and load a model in Settings."}),
                );
                return Ok(String::new());
            }
            eprintln!("[recording] Transcribing via Vosk...");
            vosk.transcribe_samples(samples, SAMPLE_RATE as f32)?.text
        }
        SttProvider::ElevenLabs => {
            eprintln!("[recording] Transcribing via ElevenLabs...");
            let wav_data = wav::encode_wav(samples, SAMPLE_RATE);
            let api_key = get_api_key(keystore, "elevenlabs")?;
            let engine = ElevenLabsEngine::new(api_key, model);
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                engine.transcribe(&wav_data, &config),
            )
            .await
            .map_err(|_| AppError::Stt("Cloud STT timeout. Check internet and try again.".into()))??
            .text
        }
        SttProvider::OpenAI => {
            eprintln!("[recording] Transcribing via OpenAI...");
            let wav_data = wav::encode_wav(samples, SAMPLE_RATE);
            let api_key = get_api_key(keystore, "openai")?;
            let engine = OpenAiWhisperEngine::new(api_key, model, base_url.clone());
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                engine.transcribe(&wav_data, &config),
            )
            .await
            .map_err(|_| AppError::Stt("Cloud STT timeout. Check internet and try again.".into()))??
            .text
        }
        SttProvider::OpenAITranscribe => {
            eprintln!("[recording] Transcribing via OpenAI Transcribe...");
            let wav_data = wav::encode_wav(samples, SAMPLE_RATE);
            let api_key = get_api_key(keystore, "openai")?;
            let transcribe_model = model.or_else(|| Some("gpt-4o-mini-transcribe".to_string()));
            let engine = OpenAiWhisperEngine::new(api_key, transcribe_model, base_url.clone());
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                engine.transcribe(&wav_data, &config),
            )
            .await
            .map_err(|_| AppError::Stt("Cloud STT timeout. Check internet and try again.".into()))??
            .text
        }
        SttProvider::OpenRouter => {
            eprintln!("[recording] Transcribing via OpenRouter Audio...");
            let wav_data = wav::encode_wav(samples, SAMPLE_RATE);
            let api_key = get_api_key(keystore, "openrouter")?;
            let engine = OpenRouterAudioEngine::new(api_key, model, base_url.clone());
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                engine.transcribe(&wav_data, &config),
            )
            .await
            .map_err(|_| AppError::Stt("Cloud STT timeout. Check internet and try again.".into()))??
            .text
        }
        SttProvider::CustomOpenAiCompatible => {
            eprintln!("[recording] Transcribing via Custom OpenAI-Compatible Audio...");
            let wav_data = wav::encode_wav(samples, SAMPLE_RATE);
            let api_key = get_api_key(keystore, "custom_openai_compatible")?;
            let endpoint = base_url.clone().ok_or_else(|| {
                AppError::Stt("Custom provider requires OpenAI-compatible endpoint.".to_string())
            })?;
            let engine = OpenRouterAudioEngine::new(api_key, model, Some(endpoint));
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                engine.transcribe(&wav_data, &config),
            )
            .await
            .map_err(|_| AppError::Stt("Cloud STT timeout. Check internet and try again.".into()))??
            .text
        }
        SttProvider::Mistral => {
            eprintln!("[recording] Transcribing via Mistral...");
            let wav_data = wav::encode_wav(samples, SAMPLE_RATE);
            let api_key = get_api_key(keystore, "mistral")?;
            let engine = MistralEngine::new(api_key, model);
            tokio::time::timeout(
                Duration::from_secs(timeout_secs),
                engine.transcribe(&wav_data, &config),
            )
            .await
            .map_err(|_| AppError::Stt("Cloud STT timeout. Check internet and try again.".into()))??
            .text
        }
    };

    if needs_s2t {
        text = converter::simplified_to_traditional(&text);
    }

    Ok(text)
}

fn get_api_key(keystore: &KeyStore, provider: &str) -> Result<String, AppError> {
    keystore
        .get_api_key(provider)?
        .ok_or_else(|| AppError::Stt(format!("No API key configured for {provider}")))
}

/// Get the default recordings output directory (auto-creates if missing).
#[tauri::command]
pub fn get_recordings_dir() -> Result<String, AppError> {
    let dir = resolve_output_dir(None)?;
    fs::create_dir_all(&dir)?;
    Ok(dir.display().to_string())
}

fn resolve_output_dir(custom: Option<String>) -> Result<PathBuf, AppError> {
    if let Some(dir) = custom {
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    // ~/Documents/Voxlore/recordings/
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| AppError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cannot determine home directory",
        )))?;

    Ok(home.join("Documents").join("Voxlore").join("recordings"))
}
