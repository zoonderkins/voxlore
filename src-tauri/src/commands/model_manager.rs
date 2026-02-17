use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::models::{downloader, registry};
use crate::stt::vosk_engine::{VoskManager, VoskModelStatus};

/// Get the models directory path (inside app data dir).
fn models_dir(app: &AppHandle) -> Result<std::path::PathBuf, AppError> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Audio(format!("Failed to resolve app data dir: {e}")))?;
    Ok(data_dir.join("models").join("vosk"))
}

/// Download a Vosk model by its ID.
#[tauri::command]
pub async fn download_vosk_model(
    app: AppHandle,
    model_id: String,
) -> Result<String, AppError> {
    let model_info = registry::available_models()
        .into_iter()
        .find(|m| m.id == model_id)
        .ok_or_else(|| AppError::Audio(format!("Unknown model: {model_id}")))?;

    let dir = models_dir(&app)?;
    let model_path = downloader::download_model(&app, &model_id, &model_info.url, &dir).await?;

    Ok(model_path.display().to_string())
}

/// Load a previously downloaded Vosk model into memory.
#[tauri::command]
pub fn load_vosk_model(
    app: AppHandle,
    model_id: String,
    vosk: State<'_, VoskManager>,
) -> Result<VoskModelStatus, AppError> {
    let dir = models_dir(&app)?;
    let model_dir = dir.join(&model_id);

    if !model_dir.exists() {
        return Err(AppError::Stt(format!(
            "Model not downloaded: {model_id}. Download it first."
        )));
    }

    vosk.load_model(&model_id, &model_dir)?;
    Ok(vosk.status())
}

/// Unload the current Vosk model from memory.
#[tauri::command]
pub fn unload_vosk_model(vosk: State<'_, VoskManager>) -> VoskModelStatus {
    vosk.unload_model();
    vosk.status()
}

/// Get the current Vosk model status.
#[tauri::command]
pub fn get_vosk_status(vosk: State<'_, VoskManager>) -> VoskModelStatus {
    vosk.status()
}

/// List models that are already downloaded on disk.
#[tauri::command]
pub fn list_downloaded_vosk_models(app: AppHandle) -> Result<Vec<String>, AppError> {
    let dir = models_dir(&app)?;
    Ok(downloader::list_downloaded_models(&dir))
}
