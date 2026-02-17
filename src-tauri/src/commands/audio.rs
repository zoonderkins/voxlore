use crate::models::registry;

/// List available Vosk models.
#[tauri::command]
pub fn list_vosk_models() -> Vec<registry::VoskModel> {
    registry::available_models()
}
