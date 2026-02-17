use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[cfg(feature = "vosk-stt")]
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::stt::SttResult;

/// Status of the Vosk model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoskModelStatus {
    pub loaded: bool,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
}

/// Manages the Vosk model lifecycle and transcription.
///
/// The model is expensive to load (can be hundreds of MB), so we load
/// it once and keep it in Tauri managed state for the app's lifetime.
pub struct VoskManager {
    #[cfg(feature = "vosk-stt")]
    model: Mutex<Option<Arc<vosk::Model>>>,
    model_id: Mutex<Option<String>>,
    model_path: Mutex<Option<PathBuf>>,
}

impl VoskManager {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "vosk-stt")]
            model: Mutex::new(None),
            model_id: Mutex::new(None),
            model_path: Mutex::new(None),
        }
    }

    /// Load a Vosk model from disk.
    pub fn load_model(&self, model_id: &str, model_dir: &Path) -> Result<(), AppError> {
        if !model_dir.exists() {
            return Err(AppError::Stt(format!(
                "Model directory does not exist: {}",
                model_dir.display()
            )));
        }

        #[cfg(feature = "vosk-stt")]
        {
            let path_str = model_dir
                .to_str()
                .ok_or_else(|| AppError::Stt("Invalid model path encoding".into()))?;

            let model = vosk::Model::new(path_str)
                .ok_or_else(|| AppError::Stt("Failed to load Vosk model".into()))?;

            *self.model.lock().unwrap() = Some(Arc::new(model));
        }

        *self.model_id.lock().unwrap() = Some(model_id.to_string());
        *self.model_path.lock().unwrap() = Some(model_dir.to_path_buf());

        Ok(())
    }

    /// Check if a model is currently loaded.
    pub fn is_loaded(&self) -> bool {
        #[cfg(feature = "vosk-stt")]
        {
            self.model.lock().unwrap().is_some()
        }
        #[cfg(not(feature = "vosk-stt"))]
        {
            false
        }
    }

    /// Get the current model status.
    pub fn status(&self) -> VoskModelStatus {
        VoskModelStatus {
            loaded: self.is_loaded(),
            model_id: self.model_id.lock().unwrap().clone(),
            model_path: self
                .model_path
                .lock()
                .unwrap()
                .as_ref()
                .map(|p| p.display().to_string()),
        }
    }

    /// Transcribe raw PCM i16 LE audio data at the given sample rate.
    ///
    /// `audio_data` is raw bytes: pairs of little-endian i16 samples.
    pub fn transcribe(&self, audio_data: &[u8], sample_rate: f32) -> Result<SttResult, AppError> {
        #[cfg(feature = "vosk-stt")]
        {
            let model = self
                .model
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| AppError::Stt("No Vosk model loaded".into()))?;

            // Convert raw bytes to i16 samples (little-endian)
            let samples: Vec<i16> = audio_data
                .chunks_exact(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();

            let mut recognizer = vosk::Recognizer::new(&model, sample_rate)
                .ok_or_else(|| AppError::Stt("Failed to create Vosk recognizer".into()))?;

            // Feed audio in chunks to allow internal processing
            const CHUNK_SIZE: usize = 4000; // ~250ms at 16kHz
            for chunk in samples.chunks(CHUNK_SIZE) {
                recognizer
                    .accept_waveform(chunk)
                    .map_err(|e| AppError::Stt(format!("Vosk waveform error: {e}")))?;
            }

            let result = recognizer.final_result();
            let text = match result.clone().single() {
                Some(r) => r.text.to_string(),
                None => match result.multiple() {
                    Some(multi) => multi
                        .alternatives
                        .first()
                        .map(|a| a.text.to_string())
                        .unwrap_or_default(),
                    None => String::new(),
                },
            };

            Ok(SttResult {
                text,
                confidence: None,
                language_detected: None,
            })
        }

        #[cfg(not(feature = "vosk-stt"))]
        {
            let _ = (audio_data, sample_rate);
            Err(AppError::Stt(
                "Vosk feature not enabled. Rebuild with --features vosk-stt".into(),
            ))
        }
    }

    /// Transcribe i16 samples directly (for use with AudioCapture output).
    pub fn transcribe_samples(
        &self,
        samples: &[i16],
        sample_rate: f32,
    ) -> Result<SttResult, AppError> {
        #[cfg(feature = "vosk-stt")]
        {
            let model = self
                .model
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| AppError::Stt("No Vosk model loaded".into()))?;

            let mut recognizer = vosk::Recognizer::new(&model, sample_rate)
                .ok_or_else(|| AppError::Stt("Failed to create Vosk recognizer".into()))?;

            const CHUNK_SIZE: usize = 4000;
            for chunk in samples.chunks(CHUNK_SIZE) {
                recognizer
                    .accept_waveform(chunk)
                    .map_err(|e| AppError::Stt(format!("Vosk waveform error: {e}")))?;
            }

            let result = recognizer.final_result();
            let text = match result.clone().single() {
                Some(r) => r.text.to_string(),
                None => match result.multiple() {
                    Some(multi) => multi
                        .alternatives
                        .first()
                        .map(|a| a.text.to_string())
                        .unwrap_or_default(),
                    None => String::new(),
                },
            };

            Ok(SttResult {
                text,
                confidence: None,
                language_detected: None,
            })
        }

        #[cfg(not(feature = "vosk-stt"))]
        {
            let _ = (samples, sample_rate);
            Err(AppError::Stt(
                "Vosk feature not enabled. Rebuild with --features vosk-stt".into(),
            ))
        }
    }

    /// Unload the current model to free memory.
    pub fn unload_model(&self) {
        #[cfg(feature = "vosk-stt")]
        {
            *self.model.lock().unwrap() = None;
        }
        *self.model_id.lock().unwrap() = None;
        *self.model_path.lock().unwrap() = None;
    }
}
