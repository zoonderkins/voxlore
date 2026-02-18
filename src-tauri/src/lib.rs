mod audio;
mod commands;
mod enhancement;
mod error;
mod hotkey;
mod logger;
mod models;
mod security;
mod stt;
mod state;
mod text_insertion;

use security::keystore::KeyStore;
use state::AppState;
use stt::vosk_engine::VoskManager;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
    Emitter, Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    logger::init_file_logger();
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::default())
        .manage(KeyStore::new())
        .manage(VoskManager::new())
        .setup(|app| {
            setup_tray(app)?;
            setup_global_shortcuts(app)?;
            auto_load_vosk_model(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // App info
            commands::greet,
            commands::get_app_info,
            // STT
            commands::stt::transcribe_audio,
            // Enhancement
            commands::enhancement::enhance_text,
            // Settings / API keys
            commands::settings::save_api_key,
            commands::settings::has_api_key,
            commands::settings::delete_api_key,
            commands::settings::sync_settings,
            commands::settings::debug_ui_event,
            commands::settings::check_provider_health,
            commands::settings::open_devtools,
            // Text insertion
            commands::text_insert::insert_text_at_cursor,
            // Audio / Models
            commands::audio::list_vosk_models,
            // Floating widget
            commands::floating::show_floating_widget,
            commands::floating::hide_floating_widget,
            // Preview
            commands::preview::show_preview_window,
            commands::preview::get_preview_text,
            commands::preview::get_preview_target_bundle_id,
            commands::preview::close_preview_window,
            commands::preview::apply_preview_text,
            // Model management
            commands::model_manager::download_vosk_model,
            commands::model_manager::load_vosk_model,
            commands::model_manager::unload_vosk_model,
            commands::model_manager::get_vosk_status,
            commands::model_manager::list_downloaded_vosk_models,
            // Permissions
            commands::permissions::check_permissions,
            commands::permissions::request_microphone_permission,
            commands::permissions::request_accessibility_permission,
            // Recording pipeline
            commands::recording::start_recording,
            commands::recording::stop_recording,
            commands::recording::get_recordings_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let show_settings = MenuItemBuilder::with_id("settings", "Settings").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit Voxlore").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_settings)
        .separator()
        .item(&quit)
        .build()?;

    let _tray = TrayIconBuilder::new()
        .tooltip("Voxlore — Ready")
        .menu(&menu)
        .on_menu_event(move |app: &tauri::AppHandle, event: tauri::menu::MenuEvent| match event.id().as_ref() {
            "settings" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Auto-load a Vosk model at startup, preferring one that matches the STT language.
fn auto_load_vosk_model(app: &tauri::App) {
    let vosk = app.state::<VoskManager>();
    if vosk.is_loaded() {
        return;
    }

    let data_dir = match app.path().app_data_dir() {
        Ok(d) => d,
        Err(_) => return,
    };
    let models_dir = data_dir.join("models").join("vosk");
    let downloaded = models::downloader::list_downloaded_models(&models_dir);

    if downloaded.is_empty() {
        crate::app_log!("[startup] No downloaded Vosk models found");
        return;
    }

    // Read the user's preferred STT language from synced state
    let lang = app.state::<AppState>().stt_language.lock().unwrap().clone();

    // Map language codes to model name fragments
    let lang_fragment = match lang.as_str() {
        "zh" => "cn",
        "ja" => "ja",
        "ko" => "ko",
        _ => "en",
    };

    // Prefer a model matching the language, fall back to first available
    let model_id = downloaded
        .iter()
        .find(|id| id.contains(lang_fragment))
        .unwrap_or(&downloaded[0]);

    let model_dir = models_dir.join(model_id);
    match vosk.load_model(model_id, &model_dir) {
        Ok(()) => crate::app_log!("[startup] Auto-loaded Vosk model: {model_id} (lang={lang})"),
        Err(e) => crate::app_log!("[startup] Failed to auto-load Vosk model: {e}"),
    }
}

/// Register global keyboard shortcuts for push-to-talk and toggle recording.
fn setup_global_shortcuts(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    {
        use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};

        app.handle().plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_shortcuts(["alt+space"])?
                .with_handler(|app, shortcut, event| {
                    // Option+Space = Push-to-talk
                    if shortcut.matches(Modifiers::ALT, Code::Space) {
                        let app_handle = app.clone();
                        match event.state {
                            ShortcutState::Pressed => {
                                crate::app_log!("[shortcut] Option+Space PRESSED");
                                tauri::async_runtime::spawn(async move {
                                    let self_bundle_id = app_handle.config().identifier.clone();
                                    let target_bundle = capture_frontmost_bundle_id_before_recording(&self_bundle_id);
                                    crate::app_log!("[shortcut] captured recording target bundle id: {:?}", target_bundle);
                                    *app_handle
                                        .state::<AppState>()
                                        .recording_target_bundle_id
                                        .lock()
                                        .unwrap() = target_bundle;

                                    // Read widget position from synced settings
                                    let app_state = app_handle.state::<AppState>();
                                    let floating_enabled = *app_state
                                        .floating_window_enabled
                                        .lock()
                                        .unwrap();
                                    let position = app_state
                                        .widget_position
                                        .lock()
                                        .unwrap()
                                        .clone();
                                    // Show floating widget only when enabled in settings
                                    if floating_enabled {
                                        if let Err(e) = commands::floating::show_floating_widget(app_handle.clone(), Some(position)).await {
                                            crate::app_log!("[shortcut] Failed to show widget: {e}");
                                        }
                                    }

                                    if let Err(e) = commands::recording::start_recording(app_handle.clone(), app_state).await {
                                        crate::app_log!("[shortcut] Failed to start recording: {e}");
                                        let _ = app_handle.emit("recording:status", serde_json::json!({"status": "error", "message": e.to_string()}));
                                    }
                                });
                            }
                            ShortcutState::Released => {
                                crate::app_log!("[shortcut] Option+Space RELEASED");
                                tauri::async_runtime::spawn(async move {
                                    let state = app_handle.state::<AppState>();
                                    let vosk = app_handle.state::<VoskManager>();
                                    let keystore = app_handle.state::<KeyStore>();

                                    match commands::recording::stop_recording(app_handle.clone(), None, state, vosk, keystore).await {
                                        Ok(result) => {
                                            crate::app_log!("[shortcut] Recording result: audio={:?}, text_len={}", result.audio_path, result.text.len());
                                            // Hide floating widget
                                            let _ = commands::floating::hide_floating_widget(app_handle.clone()).await;

                                            if !result.text.is_empty() {
                                                // Emit result to frontend so it can decide: preview or insert
                                                let _ = app_handle.emit("recording:result", &result);
                                            }
                                        }
                                        Err(e) => {
                                            let _ = commands::floating::hide_floating_widget(app_handle.clone()).await;
                                            crate::app_log!("[shortcut] Failed to stop recording: {e}");
                                            let _ = app_handle.emit("recording:status", serde_json::json!({"status": "error", "message": e.to_string()}));
                                        }
                                    }
                                });
                            }
                        }
                    }
                })
                .build(),
        )?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn capture_frontmost_bundle_id_before_recording(self_bundle_id: &str) -> Option<String> {
    use std::ffi::{c_void, CStr};
    use std::os::raw::c_char;

    type Id = *mut c_void;
    type Sel = *mut c_void;

    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    type MsgSendId = unsafe extern "C" fn(Id, Sel) -> Id;
    type MsgSendCStr = unsafe extern "C" fn(Id, Sel) -> *const c_char;

    unsafe {
        let send_id: MsgSendId = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_cstr: MsgSendCStr = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());

        let workspace_cls = objc_getClass(b"NSWorkspace\0".as_ptr() as *const c_char);
        if workspace_cls.is_null() {
            return None;
        }
        let shared_workspace_sel = sel_registerName(b"sharedWorkspace\0".as_ptr() as *const c_char);
        let frontmost_app_sel =
            sel_registerName(b"frontmostApplication\0".as_ptr() as *const c_char);
        let bundle_identifier_sel =
            sel_registerName(b"bundleIdentifier\0".as_ptr() as *const c_char);
        let utf8_string_sel = sel_registerName(b"UTF8String\0".as_ptr() as *const c_char);

        // 熱鍵事件觸發瞬間可能會有前景切換競態，做短暫輪詢提高命中率。
        let start = std::time::Instant::now();
        while start.elapsed().as_millis() < 500 {
            let workspace = send_id(workspace_cls, shared_workspace_sel);
            if workspace.is_null() {
                std::thread::sleep(std::time::Duration::from_millis(25));
                continue;
            }
            let app = send_id(workspace, frontmost_app_sel);
            if app.is_null() {
                std::thread::sleep(std::time::Duration::from_millis(25));
                continue;
            }
            let bundle_ns = send_id(app, bundle_identifier_sel);
            if bundle_ns.is_null() {
                std::thread::sleep(std::time::Duration::from_millis(25));
                continue;
            }
            let c_str = send_cstr(bundle_ns, utf8_string_sel);
            if c_str.is_null() {
                std::thread::sleep(std::time::Duration::from_millis(25));
                continue;
            }
            let bundle_id = CStr::from_ptr(c_str).to_string_lossy().to_string();
            if !bundle_id.is_empty() && bundle_id != self_bundle_id {
                return Some(bundle_id);
            }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
        None
    }
}

#[cfg(not(target_os = "macos"))]
fn capture_frontmost_bundle_id_before_recording(_self_bundle_id: &str) -> Option<String> {
    None
}
