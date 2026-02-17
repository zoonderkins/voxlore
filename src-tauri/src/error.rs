use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("STT error: {0}")]
    Stt(String),

    #[error("Enhancement error: {0}")]
    Enhancement(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Text insertion error: {0}")]
    TextInsertion(String),

    #[error("Security error: {0}")]
    Security(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
