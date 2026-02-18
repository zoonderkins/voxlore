use std::ffi::c_void;
use std::process::Command;

use crate::error::AppError;

// CoreGraphics FFI for keyboard event simulation
type CGEventRef = *mut c_void;
type CGEventSourceRef = *const c_void;

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventCreateKeyboardEvent(
        source: CGEventSourceRef,
        keycode: u16,
        key_down: bool,
    ) -> CGEventRef;
    fn CGEventSetFlags(event: CGEventRef, flags: u64);
    fn CGEventPost(tap: u32, event: CGEventRef);
    fn CGPreflightPostEventAccess() -> bool;
    fn CGRequestPostEventAccess() -> bool;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;
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
    static kCFBooleanTrue: *const c_void;
}

// kVK_ANSI_V = 0x09
const VK_ANSI_V: u16 = 0x09;
// kCGEventFlagMaskCommand = NX_COMMANDMASK = 1 << 20
const CG_EVENT_FLAG_MASK_COMMAND: u64 = 1 << 20;
// kCGHIDEventTap = 0
const CG_HID_EVENT_TAP: u32 = 0;

/// Insert text at cursor on macOS using clipboard + Cmd+V simulation.
///
/// Strategy: always copy text to clipboard, then attempt CGEvent Cmd+V.
/// - With Accessibility: auto-paste works, clipboard is restored afterward.
/// - Without Accessibility: CGEvent is silently dropped by macOS, but text
///   remains on the clipboard so the user can Cmd+V manually.
///
/// This "graceful degradation" approach avoids error dialogs and always
/// leaves the text accessible to the user.
/// Returns `Ok(true)` if auto-pasted, `Ok(false)` if text is on clipboard only.
pub async fn insert_text(text: &str) -> Result<bool, AppError> {
    crate::app_log!("[text-insert] Inserting {} chars", text.len());
    crate::app_log!(
        "[text-insert] frontmost before insert: {:?}",
        get_frontmost_bundle_id()
    );

    // Save current clipboard content
    let saved = get_clipboard();

    // Set clipboard to our text
    set_clipboard(text)?;

    // Small delay to ensure clipboard is ready
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let trusted = unsafe { AXIsProcessTrusted() };
    let mut post_event_allowed = unsafe { CGPreflightPostEventAccess() };
    if !trusted {
        let prompt_requested = request_accessibility_trust_prompt();
        crate::app_log!("[text-insert] requested AX trust prompt: {prompt_requested}");
    }
    if !post_event_allowed {
        let _ = unsafe { CGRequestPostEventAccess() };
        // 使用者剛在系統彈窗點允許時，狀態更新常會稍有延遲。
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        post_event_allowed = unsafe { CGPreflightPostEventAccess() };
    }
    crate::app_log!(
        "[text-insert] AXIsProcessTrusted = {trusted}, CGPostEventAccess = {post_event_allowed}"
    );

    // 不因前置權限判斷直接中止，仍嘗試送出貼上事件。
    // 某些環境下 preflight 可能回傳 false，但實際事件仍可能成功。
    let untrusted_mode = !(trusted && post_event_allowed);
    if untrusted_mode {
        crate::app_log!("[text-insert] Event permission preflight not fully granted; still attempting paste");
    }

    let mut pasted = match simulate_cmd_v() {
        Ok(()) => {
            crate::app_log!("[text-insert] CGEvent Cmd+V posted");
            true
        }
        Err(e) => {
            crate::app_log!("[text-insert] CGEvent failed: {e}");
            false
        }
    };

    if !pasted {
        crate::app_log!("[text-insert] Falling back to osascript System Events Cmd+V");
        pasted = match simulate_cmd_v_osascript() {
            Ok(()) => {
                crate::app_log!("[text-insert] osascript Cmd+V posted");
                true
            }
            Err(e) => {
                crate::app_log!("[text-insert] osascript Cmd+V failed: {e}");
                false
            }
        };
    }

    // Wait for the paste to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    if pasted && !untrusted_mode {
        // Paste succeeded — restore original clipboard
        if let Some(saved) = saved {
            let _ = set_clipboard(&saved);
        }
        crate::app_log!("[text-insert] Clipboard restored");
        crate::app_log!(
            "[text-insert] frontmost after insert: {:?}",
            get_frontmost_bundle_id()
        );
        Ok(true)
    } else if pasted && untrusted_mode {
        // 權限狀態不完整時，無法可靠判斷是否真的貼入，保留剪貼簿內容供手動 Cmd+V。
        crate::app_log!(
            "[text-insert] Cmd+V posted in untrusted mode; keeping clipboard text for manual paste"
        );
        crate::app_log!(
            "[text-insert] frontmost after insert attempt: {:?}",
            get_frontmost_bundle_id()
        );
        Ok(false)
    } else {
        // Paste didn't go through — keep text on clipboard for manual Cmd+V
        crate::app_log!("[text-insert] Text left on clipboard (Cmd+V manually if needed)");
        crate::app_log!(
            "[text-insert] frontmost after failed insert: {:?}",
            get_frontmost_bundle_id()
        );
        Ok(false)
    }
}

fn get_clipboard() -> Option<String> {
    Command::new("pbpaste")
        .env("LANG", "en_US.UTF-8")
        .env("LC_CTYPE", "en_US.UTF-8")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
}

fn set_clipboard(text: &str) -> Result<(), AppError> {
    let mut child = Command::new("pbcopy")
        .env("LANG", "en_US.UTF-8")
        .env("LC_CTYPE", "en_US.UTF-8")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| AppError::TextInsertion(format!("Failed to set clipboard: {e}")))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| AppError::TextInsertion(format!("Failed to write to clipboard: {e}")))?;
    }
    child
        .wait()
        .map_err(|e| AppError::TextInsertion(format!("pbcopy failed: {e}")))?;
    Ok(())
}

fn simulate_cmd_v() -> Result<(), AppError> {
    unsafe {
        // Key down
        let key_down = CGEventCreateKeyboardEvent(std::ptr::null(), VK_ANSI_V, true);
        if key_down.is_null() {
            return Err(AppError::TextInsertion(
                "Failed to create key-down event".into(),
            ));
        }
        CGEventSetFlags(key_down, CG_EVENT_FLAG_MASK_COMMAND);
        CGEventPost(CG_HID_EVENT_TAP, key_down);
        CFRelease(key_down as *const c_void);

        // Key up
        let key_up = CGEventCreateKeyboardEvent(std::ptr::null(), VK_ANSI_V, false);
        if !key_up.is_null() {
            CGEventSetFlags(key_up, CG_EVENT_FLAG_MASK_COMMAND);
            CGEventPost(CG_HID_EVENT_TAP, key_up);
            CFRelease(key_up as *const c_void);
        }
    }
    Ok(())
}

fn simulate_cmd_v_osascript() -> Result<(), AppError> {
    let script = "tell application \"System Events\" to keystroke \"v\" using command down";
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| AppError::TextInsertion(format!("osascript launch failed: {e}")))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(AppError::TextInsertion(format!(
            "osascript returned non-zero: {stderr}"
        )))
    }
}

fn get_frontmost_bundle_id() -> Option<String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get bundle identifier of first process whose frontmost is true")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let bundle_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if bundle_id.is_empty() {
        None
    } else {
        Some(bundle_id)
    }
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
