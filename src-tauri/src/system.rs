//! 설정 창 명령: 권한, 로그인 항목, 단축키 검증.

use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub accessibility_trusted: bool,
    pub login_item: layout_macos::login_item::LoginItemStatus,
    pub hotkey_failures: Vec<crate::hotkeys::HotkeyFailure>,
    pub settings_load_error: Option<String>,
}

pub fn status(app: &AppHandle) -> Status {
    Status {
        accessibility_trusted: layout_macos::permission::is_trusted(),
        login_item: layout_macos::login_item::status(),
        hotkey_failures: app.state::<crate::hotkeys::HotkeyState>().failures(),
        settings_load_error: app.state::<crate::settings::SettingsState>().load_error(),
    }
}

#[tauri::command]
pub fn status_get(app: AppHandle) -> Status {
    status(&app)
}

#[tauri::command]
pub fn permission_prompt(app: AppHandle) -> Status {
    layout_macos::permission::prompt();
    status(&app)
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    layout_macos::permission::open_settings().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn login_item_set(app: AppHandle, enabled: bool) -> Result<Status, String> {
    layout_macos::login_item::set_enabled(enabled)?;
    crate::tray::refresh_status(&app);
    Ok(status(&app))
}

#[tauri::command]
pub fn open_login_items_settings() {
    layout_macos::login_item::open_settings();
}

/// 단축키 문자열을 진짜 파서로 검증한다. 성공하면 정규화된 표기를 돌려준다.
#[tauri::command]
pub fn shortcut_validate(spec: String) -> Result<String, String> {
    layout_core::settings::check_shortcut_shape(&spec)?;
    crate::hotkeys::validate(&spec)
}
