pub mod audio;
pub mod enhancement;
pub mod floating;
pub mod model_manager;
pub mod permissions;
pub mod preview;
pub mod recording;
pub mod settings;
pub mod stt;
pub mod text_insert;

use serde::Serialize;

#[derive(Serialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to Voxlore.", name)
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Voxlore".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
