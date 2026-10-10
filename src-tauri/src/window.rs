//! 설정 창(label `settings`): 트레이 "설정…"에서 지연 생성, 닫으면 숨김.

use serde::Serialize;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_opener::OpenerExt;

pub const SETTINGS_LABEL: &str = "settings";

pub fn show_settings(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }
    let built = WebviewWindowBuilder::new(app, SETTINGS_LABEL, WebviewUrl::default())
        .title("IYAGI Layout")
        .inner_size(560.0, 720.0)
        .min_inner_size(480.0, 560.0)
        .resizable(true)
        .build();
    match built {
        Ok(window) => {
            let _ = window.set_focus();
        }
        Err(error) => tracing::error!(%error, "cannot create the settings window"),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub log_dir: Option<String>,
    /// 시스템 언어로 정한 UI 언어(`en|ko|ja|zh`).
    pub lang: layout_core::i18n::Lang,
}

#[tauri::command]
pub fn app_info(app: AppHandle) -> AppInfo {
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        log_dir: crate::logging::log_dir(&app).map(|p| p.display().to_string()),
        lang: crate::i18n::current(),
    }
}

#[tauri::command]
pub fn open_log_dir(app: AppHandle) -> Result<(), String> {
    let dir = crate::logging::log_dir(&app).ok_or("log directory unavailable")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(dir.display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}
