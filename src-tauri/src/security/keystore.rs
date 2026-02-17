use crate::error::AppError;

const SERVICE_NAME: &str = "app.voxlore";

/// OS-native keychain storage for API keys.
/// Uses macOS Keychain, Windows Credential Manager, or Linux Secret Service.
pub struct KeyStore;

impl KeyStore {
    pub fn new() -> Self {
        Self
    }

    pub fn save_api_key(&self, provider: &str, key: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider)
            .map_err(|e| AppError::Security(format!("Keyring entry error: {e}")))?;
        entry
            .set_password(key)
            .map_err(|e| AppError::Security(format!("Failed to save key for {provider}: {e}")))
    }

    pub fn get_api_key(&self, provider: &str) -> Result<Option<String>, AppError> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider)
            .map_err(|e| AppError::Security(format!("Keyring entry error: {e}")))?;
        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::Security(format!(
                "Failed to get key for {provider}: {e}"
            ))),
        }
    }

    pub fn delete_api_key(&self, provider: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider)
            .map_err(|e| AppError::Security(format!("Keyring entry error: {e}")))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(AppError::Security(format!(
                "Failed to delete key for {provider}: {e}"
            ))),
        }
    }

    pub fn has_api_key(&self, provider: &str) -> Result<bool, AppError> {
        let entry = keyring::Entry::new(SERVICE_NAME, provider)
            .map_err(|e| AppError::Security(format!("Keyring entry error: {e}")))?;
        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(AppError::Security(format!(
                "Failed to check key for {provider}: {e}"
            ))),
        }
    }
}
