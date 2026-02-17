use crate::error::AppError;

/// Insert text at cursor on Windows.
/// TODO: Implement using SendInput API or UI Automation.
pub async fn insert_text(_text: &str) -> Result<(), AppError> {
    Err(AppError::TextInsertion(
        "Windows text insertion not yet implemented".to_string(),
    ))
}
