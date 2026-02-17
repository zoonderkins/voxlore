use crate::error::AppError;
use crate::text_insertion;

/// Insert text at the current cursor position.
/// Returns `true` if auto-pasted, `false` if text is on clipboard only.
#[tauri::command]
pub async fn insert_text_at_cursor(text: String) -> Result<bool, AppError> {
    text_insertion::insert_text_at_cursor(&text).await
}
