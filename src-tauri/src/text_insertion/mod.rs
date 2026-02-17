#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

use crate::error::AppError;

/// Insert text at the current cursor position in any application.
/// Returns `Ok(true)` if auto-pasted, `Ok(false)` if clipboard-only.
pub async fn insert_text_at_cursor(text: &str) -> Result<bool, AppError> {
    #[cfg(target_os = "macos")]
    return macos::insert_text(text).await;

    #[cfg(target_os = "windows")]
    return windows::insert_text(text).await;

    #[cfg(target_os = "linux")]
    return linux::insert_text(text).await;

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    Err(AppError::TextInsertion(
        "Unsupported platform".to_string(),
    ))
}
