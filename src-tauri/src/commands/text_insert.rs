use tauri::{AppHandle, State};

use crate::error::AppError;
use crate::state::AppState;
use crate::text_insertion;

/// Insert text at the current cursor position.
/// Returns `true` if auto-pasted, `false` if text is on clipboard only.
#[tauri::command]
pub async fn insert_text_at_cursor(
    app: AppHandle,
    state: State<'_, AppState>,
    text: String,
) -> Result<bool, AppError> {
    let self_bundle_id = app.config().identifier.clone();
    crate::app_log!("[insert] app bundle identifier: {self_bundle_id}");
    if let Ok(exe) = std::env::current_exe() {
        crate::app_log!("[insert] current executable: {}", exe.display());
    }
    let target_bundle = state
        .recording_target_bundle_id
        .lock()
        .unwrap()
        .clone()
        .filter(|bundle_id| bundle_id != &self_bundle_id)
        .or_else(|| wait_for_non_self_frontmost_bundle_id(&self_bundle_id, 800, 40));

    if let Some(bundle_id) = target_bundle.as_deref() {
        crate::app_log!("[insert] re-activating target app before direct insert: {bundle_id}");
        if let Err(e) = activate_app_by_bundle_id(bundle_id) {
            crate::app_log!("[insert] failed to activate target app: {e}");
        }
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    }

    let mut auto_pasted = text_insertion::insert_text_at_cursor(&text).await?;
    if !auto_pasted {
        crate::app_log!("[insert] first direct insert attempt returned clipboard-only, retrying once");
        if let Some(bundle_id) = target_bundle.as_deref() {
            let _ = activate_app_by_bundle_id(bundle_id);
        }
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        auto_pasted = text_insertion::insert_text_at_cursor(&text).await?;
    }

    if let Some(bundle_id) = target_bundle.as_deref() {
        crate::app_log!("[insert] restoring focus back to target app after insert flow: {bundle_id}");
        let _ = activate_app_by_bundle_id(bundle_id);
    }

    Ok(auto_pasted)
}

fn activate_app_by_bundle_id(bundle_id: &str) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        activate_app_by_bundle_id_native(bundle_id)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bundle_id;
        Ok(())
    }
}

fn wait_for_non_self_frontmost_bundle_id(
    self_bundle_id: &str,
    timeout_ms: u64,
    poll_interval_ms: u64,
) -> Option<String> {
    let start = std::time::Instant::now();
    loop {
        if let Some(bundle_id) = get_frontmost_bundle_id_native() {
            if bundle_id != self_bundle_id {
                return Some(bundle_id);
            }
        }
        if start.elapsed().as_millis() >= timeout_ms as u128 {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(poll_interval_ms));
    }
}

#[cfg(target_os = "macos")]
fn get_frontmost_bundle_id_native() -> Option<String> {
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

        let workspace = send_id(workspace_cls, shared_workspace_sel);
        if workspace.is_null() {
            return None;
        }
        let app = send_id(workspace, frontmost_app_sel);
        if app.is_null() {
            return None;
        }
        let bundle_ns = send_id(app, bundle_identifier_sel);
        if bundle_ns.is_null() {
            return None;
        }
        let c_str = send_cstr(bundle_ns, utf8_string_sel);
        if c_str.is_null() {
            return None;
        }
        Some(CStr::from_ptr(c_str).to_string_lossy().to_string())
    }
}

#[cfg(not(target_os = "macos"))]
fn get_frontmost_bundle_id_native() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn activate_app_by_bundle_id_native(bundle_id: &str) -> Result<(), AppError> {
    use std::ffi::c_void;
    use std::os::raw::c_char;

    type Id = *mut c_void;
    type Sel = *mut c_void;

    extern "C" {
        fn objc_getClass(name: *const c_char) -> Id;
        fn sel_registerName(name: *const c_char) -> Sel;
        fn objc_msgSend();
    }

    type MsgSendIdStr = unsafe extern "C" fn(Id, Sel, *const c_char) -> Id;
    type MsgSendIdId = unsafe extern "C" fn(Id, Sel, Id) -> Id;
    type MsgSendU64 = unsafe extern "C" fn(Id, Sel) -> u64;
    type MsgSendIdU64 = unsafe extern "C" fn(Id, Sel, u64) -> Id;
    type MsgSendBoolU64 = unsafe extern "C" fn(Id, Sel, u64) -> bool;

    unsafe {
        let send_id_str: MsgSendIdStr =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_id_id: MsgSendIdId = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_u64: MsgSendU64 = std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_id_u64: MsgSendIdU64 =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());
        let send_bool_u64: MsgSendBoolU64 =
            std::mem::transmute(objc_msgSend as unsafe extern "C" fn());

        let ns_string_cls = objc_getClass(b"NSString\0".as_ptr() as *const c_char);
        let string_with_utf8_sel =
            sel_registerName(b"stringWithUTF8String:\0".as_ptr() as *const c_char);
        let bundle_ns = send_id_str(
            ns_string_cls,
            string_with_utf8_sel,
            format!("{bundle_id}\0").as_ptr() as *const c_char,
        );
        if bundle_ns.is_null() {
            return Err(AppError::TextInsertion(
                "Failed to build bundle id string".into(),
            ));
        }

        let running_app_cls = objc_getClass(b"NSRunningApplication\0".as_ptr() as *const c_char);
        if running_app_cls.is_null() {
            return Err(AppError::TextInsertion(
                "NSRunningApplication class not found".into(),
            ));
        }

        let running_apps_sel = sel_registerName(
            b"runningApplicationsWithBundleIdentifier:\0".as_ptr() as *const c_char,
        );
        let apps = send_id_id(running_app_cls, running_apps_sel, bundle_ns);
        if apps.is_null() {
            return Err(AppError::TextInsertion(
                "No running applications array returned".into(),
            ));
        }

        let count_sel = sel_registerName(b"count\0".as_ptr() as *const c_char);
        let count = send_u64(apps, count_sel);
        if count == 0 {
            return Err(AppError::TextInsertion(format!(
                "Target app not running: {bundle_id}"
            )));
        }

        let object_at_index_sel = sel_registerName(b"objectAtIndex:\0".as_ptr() as *const c_char);
        let app = send_id_u64(apps, object_at_index_sel, 0);
        if app.is_null() {
            return Err(AppError::TextInsertion(
                "Failed to get target app instance".into(),
            ));
        }

        let activate_sel = sel_registerName(b"activateWithOptions:\0".as_ptr() as *const c_char);
        let activated = send_bool_u64(app, activate_sel, 1);
        if !activated {
            return Err(AppError::TextInsertion(format!(
                "Failed to activate app: {bundle_id}"
            )));
        }
    }

    Ok(())
}
