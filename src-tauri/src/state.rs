use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct AppState {
    /// Signal to stop the recording background task.
    pub stop_signal: Mutex<Option<Arc<AtomicBool>>>,
    /// Handle to the background task collecting audio samples.
    pub recording_task: Mutex<Option<tokio::task::JoinHandle<Vec<i16>>>>,
    /// Text to display in the preview window (set before opening, pulled by preview on mount).
    pub preview_text: Mutex<Option<String>>,
    /// Floating widget position synced from frontend settings.
    pub widget_position: Mutex<String>,
    /// Whether floating widget is enabled.
    pub floating_window_enabled: Mutex<bool>,
    /// STT language synced from frontend settings (e.g. "en", "zh").
    pub stt_language: Mutex<String>,
    /// STT provider synced from frontend settings (e.g. "vosk", "openai").
    pub stt_provider: Mutex<String>,
    /// Optional STT model synced from frontend settings.
    pub stt_model: Mutex<Option<String>>,
    /// Optional STT OpenAI-compatible endpoint synced from frontend settings.
    pub stt_base_url: Mutex<Option<String>>,
    /// Cloud STT timeout seconds synced from frontend settings.
    pub cloud_timeout_secs: Mutex<u64>,
    /// Frontend debug logging switch.
    pub debug_logging_enabled: Mutex<bool>,
    /// Preview 開啟前的前景 App bundle id，用於 Apply 時還原焦點。
    pub preview_target_bundle_id: Mutex<Option<String>>,
    /// 熱鍵按下開始錄音時的目標 App bundle id。
    pub recording_target_bundle_id: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            stop_signal: Mutex::new(None),
            recording_task: Mutex::new(None),
            preview_text: Mutex::new(None),
            widget_position: Mutex::new("bottom-right".into()),
            floating_window_enabled: Mutex::new(false),
            stt_language: Mutex::new("en".into()),
            stt_provider: Mutex::new("vosk".into()),
            stt_model: Mutex::new(None),
            stt_base_url: Mutex::new(None),
            cloud_timeout_secs: Mutex::new(45),
            debug_logging_enabled: Mutex::new(true),
            preview_target_bundle_id: Mutex::new(None),
            recording_target_bundle_id: Mutex::new(None),
        }
    }
}
