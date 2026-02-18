use crate::error::AppError;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    pub microphone: String,
    pub accessibility: String,
}

/// Check current permission status for microphone and accessibility.
#[tauri::command]
pub async fn check_permissions() -> Result<PermissionStatus, AppError> {
    tokio::task::spawn_blocking(|| PermissionStatus {
        microphone: platform::check_microphone(),
        accessibility: platform::check_accessibility(),
    })
    .await
    .map_err(|e| AppError::Audio(format!("Permission check failed: {e}")))
}

/// Request microphone permission.
/// If "not_determined" → trigger the macOS prompt via audio device access.
/// If "denied" → open System Settings > Privacy > Microphone.
#[tauri::command]
pub async fn request_microphone_permission() -> Result<bool, AppError> {
    tokio::task::spawn_blocking(|| {
        let status = platform::check_microphone();
        if status == "denied" || status == "restricted" {
            platform::open_microphone_settings();
            false
        } else {
            platform::request_microphone()
        }
    })
    .await
    .map_err(|e| AppError::Audio(format!("Microphone request failed: {e}")))
}

/// Open macOS Accessibility settings for the user to grant access.
#[tauri::command]
pub async fn request_accessibility_permission() -> Result<(), AppError> {
    platform::request_accessibility();
    Ok(())
}

/// Check microphone permission status (for use by other modules).
pub fn microphone_status() -> String {
    platform::check_microphone()
}

/// Request microphone access (for use by other modules).
/// Returns true if access was granted.
pub fn request_microphone_access() -> bool {
    let status = platform::check_microphone();
    match status.as_str() {
        "granted" => true,
        "not_determined" => platform::request_microphone(),
        _ => false,
    }
}

// ── macOS implementation ─────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod platform {
    use std::ffi::c_void;
    use std::os::raw::c_char;

    type Id = *mut c_void;
    type Sel = *mut c_void;

    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        // NOTE: Do NOT declare objc_msgSend as variadic (...) — on ARM64
        // variadic and non-variadic functions use different calling conventions.
        // Instead, we transmute to typed function pointers below.
        fn objc_msgSend();
    }

    // Typed function pointers for specific ObjC message signatures.
    // This ensures correct ARM64 calling convention (arguments in registers, not stack).
    type MsgSendIdStr = unsafe extern "C" fn(Id, Sel, *const c_char) -> Id;
    type MsgSendIdId = unsafe extern "C" fn(Id, Sel, Id) -> i64;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
        static kAXTrustedCheckOptionPrompt: *const c_void;
    }

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGPreflightPostEventAccess() -> bool;
        fn CGRequestPostEventAccess() -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFDictionaryCreate(
            allocator: *const c_void,
            keys: *const *const c_void,
            values: *const *const c_void,
            num_values: isize,
            key_callbacks: *const c_void,
            value_callbacks: *const c_void,
        ) -> *const c_void;
        fn CFRelease(cf: *const c_void);
        static kCFBooleanTrue: *const c_void;
    }

    /// Check microphone authorization via AVCaptureDevice.authorizationStatusForMediaType:.
    /// Returns: "not_determined" | "restricted" | "denied" | "granted" | "unknown"
    pub fn check_microphone() -> String {
        unsafe {
            let cls = objc_getClass(b"AVCaptureDevice\0".as_ptr() as *const c_char);
            if cls.is_null() {
                return "unknown".into();
            }

            // Cast objc_msgSend to typed function pointers
            let send_str: MsgSendIdStr = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
            let send_id: MsgSendIdId = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());

            // [NSString stringWithUTF8String:"soun"]
            let ns_string = objc_getClass(b"NSString\0".as_ptr() as *const c_char);
            let str_sel =
                sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const c_char);
            let audio_type = send_str(ns_string, str_sel, b"soun\0".as_ptr() as *const c_char);

            // [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeAudio]
            let auth_sel = sel_registerName(
                b"authorizationStatusForMediaType:\0".as_ptr() as *const c_char,
            );
            let status = send_id(cls, auth_sel, audio_type as Id);

            match status {
                0 => "not_determined",
                1 => "restricted",
                2 => "denied",
                3 => "granted",
                _ => "unknown",
            }
            .into()
        }
    }

    /// Check accessibility via AXIsProcessTrusted().
    pub fn check_accessibility() -> String {
        let ax_trusted = unsafe { AXIsProcessTrusted() };
        let post_event_allowed = unsafe { CGPreflightPostEventAccess() };
        if ax_trusted && post_event_allowed {
            "granted".into()
        } else {
            "denied".into()
        }
    }

    /// Trigger macOS microphone permission prompt by opening and playing an audio input stream.
    pub fn request_microphone() -> bool {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

        let host = cpal::default_host();
        let Some(device) = host.default_input_device() else {
            open_microphone_settings();
            return false;
        };
        let Ok(config) = device.default_input_config() else {
            open_microphone_settings();
            return false;
        };

        match device.build_input_stream(
            &config.into(),
            |_: &[f32], _: &cpal::InputCallbackInfo| {},
            |_| {},
            None,
        ) {
            Ok(stream) => {
                // Must call play() to actually start audio capture — this triggers the macOS prompt
                let _ = stream.play();
                // Keep stream alive so macOS permission dialog can appear and user can respond
                std::thread::sleep(std::time::Duration::from_millis(500));
                drop(stream);
                true
            }
            Err(_) => {
                // If stream fails, fall back to opening System Settings
                open_microphone_settings();
                false
            }
        }
    }

    /// Open System Settings > Privacy > Accessibility.
    pub fn request_accessibility() {
        let prompt_shown = request_accessibility_trust_prompt();
        crate::app_log!("[permissions] AX prompt requested: {prompt_shown}");
        let _ = unsafe { CGRequestPostEventAccess() };
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn();
    }

    /// Open System Settings > Privacy > Microphone.
    pub fn open_microphone_settings() {
        let _ = std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
            .spawn();
    }

    fn request_accessibility_trust_prompt() -> bool {
        unsafe {
            let keys = [kAXTrustedCheckOptionPrompt];
            let values = [kCFBooleanTrue];
            let options = CFDictionaryCreate(
                std::ptr::null(),
                keys.as_ptr(),
                values.as_ptr(),
                1,
                std::ptr::null(),
                std::ptr::null(),
            );
            if options.is_null() {
                return false;
            }
            let trusted = AXIsProcessTrustedWithOptions(options);
            CFRelease(options);
            trusted
        }
    }
}

// ── Non-macOS fallback ───────────────────────────────────────────────

#[cfg(not(target_os = "macos"))]
mod platform {
    pub fn check_microphone() -> String {
        "granted".into()
    }
    pub fn check_accessibility() -> String {
        "granted".into()
    }
    pub fn request_microphone() -> bool {
        true
    }
    pub fn request_accessibility() {}
    pub fn open_microphone_settings() {}
}
