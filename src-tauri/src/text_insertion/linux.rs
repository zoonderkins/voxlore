use crate::error::AppError;

/// Insert text at cursor on Linux.
/// TODO: Implement using xdotool/ydotool or AT-SPI2.
pub async fn insert_text(_text: &str) -> Result<(), AppError> {
    Err(AppError::TextInsertion(
        "Linux text insertion not yet implemented".to_string(),
    ))
}
