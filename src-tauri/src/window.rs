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

/// 설정 창 높이를 내용에 맞춘다(너비는 그대로). 화면보다 커지지 않게 작업 영역 안으로 자른다.
#[tauri::command]
pub fn fit_settings_height(app: AppHandle, height: f64) {
    let Some(window) = app.get_webview_window(SETTINGS_LABEL) else {
        return;
    };
    let Ok(scale) = window.scale_factor() else {
        return;
    };
    let Ok(size) = window.inner_size() else {
        return;
    };
    let width = f64::from(size.width) / scale;
    let screen_height = window
        .current_monitor()
        .ok()
        .flatten()
        .map(|m| f64::from(m.size().height) / m.scale_factor());
    let target = settings_height(height, screen_height);
    let _ = window.set_size(tauri::LogicalSize::new(width, target));
}

/// 요청 높이를 최소 높이와 화면(메뉴 막대·Dock 여유 제외) 높이 사이로 자른다.
/// 화면 높이를 모르면 위쪽 한계 없이 요청값을 따른다.
fn settings_height(requested: f64, screen_height: Option<f64>) -> f64 {
    let ceiling = screen_height
        .map(|h| (h - MENU_BAR_AND_DOCK_ALLOWANCE).max(MIN_SETTINGS_HEIGHT))
        .unwrap_or(f64::INFINITY);
    requested.ceil().clamp(MIN_SETTINGS_HEIGHT, ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_is_rounded_up_within_bounds() {
        assert_eq!(settings_height(700.2, None), 701.0);
    }

    #[test]
    fn height_never_drops_below_the_minimum() {
        assert_eq!(settings_height(100.0, Some(900.0)), MIN_SETTINGS_HEIGHT);
    }

    #[test]
    fn height_stops_at_the_screen_minus_menu_bar_and_dock() {
        assert_eq!(settings_height(1200.0, Some(900.0)), 780.0);
    }

    #[test]
    fn tiny_screen_still_allows_the_minimum() {
        assert_eq!(settings_height(900.0, Some(500.0)), MIN_SETTINGS_HEIGHT);
    }
}

/// 메뉴 막대와 Dock이 차지하는 여유. 창이 화면 밖으로 나가지 않게 뺀다.
const MENU_BAR_AND_DOCK_ALLOWANCE: f64 = 120.0;
/// `min_inner_size`와 맞춘 최소 높이.
const MIN_SETTINGS_HEIGHT: f64 = 560.0;

/// 설정 창의 "메뉴 막대로 숨기기". 창의 빨간 닫기 버튼과 같다: 창만 숨기고 앱은 메뉴 막대에 남는다.
#[tauri::command]
pub fn hide_settings(app: AppHandle) {
    if let Some(window) = app.get_webview_window(SETTINGS_LABEL) {
        let _ = window.hide();
    }
    tracing::info!("settings window hidden — the app stays in the menu bar");
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
