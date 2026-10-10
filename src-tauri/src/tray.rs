//! 메뉴 막대 아이콘과 메뉴. 설정은 설정 창에서 바꾸고, 여기는 진입점과 상태만 둔다.
//! 메뉴 문구는 `layout_core::i18n`(시스템 언어 기준)에서 온다.

use std::sync::Mutex;

use layout_core::i18n::Msg;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{App, AppHandle, Manager};

pub const TRAY_ID: &str = "main";

/// 동적으로 갱신하는 메뉴 항목 핸들.
pub struct TrayHandles {
    status: MenuItem<tauri::Wry>,
    login: CheckMenuItem<tauri::Wry>,
}

pub struct TrayState(Mutex<Option<TrayHandles>>);

pub fn setup(app: &App) -> tauri::Result<()> {
    let handle = app.handle();
    let status = MenuItem::with_id(handle, "status", status_text(handle), false, None::<&str>)?;
    let settings = MenuItem::with_id(
        handle,
        "settings",
        crate::i18n::t(Msg::Settings),
        true,
        Some("Cmd+,"),
    )?;
    let permission = MenuItem::with_id(
        handle,
        "permission",
        crate::i18n::t(Msg::AccessibilitySettings),
        true,
        None::<&str>,
    )?;
    let login = CheckMenuItem::with_id(
        handle,
        "login",
        crate::i18n::t(Msg::LaunchAtLogin),
        true,
        layout_macos::login_item::status().is_enabled(),
        None::<&str>,
    )?;
    let logs = MenuItem::with_id(
        handle,
        "logs",
        crate::i18n::t(Msg::OpenLogs),
        true,
        None::<&str>,
    )?;
    let about = MenuItem::with_id(
        handle,
        "about",
        format!("IYAGI Layout {}", env!("CARGO_PKG_VERSION")),
        false,
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(
        handle,
        "quit",
        crate::i18n::t(Msg::Quit),
        true,
        Some("Cmd+Q"),
    )?;
    let menu = Menu::with_items(
        handle,
        &[
            &status,
            &PredefinedMenuItem::separator(handle)?,
            &settings,
            &permission,
            &login,
            &logs,
            &PredefinedMenuItem::separator(handle)?,
            &about,
            &quit,
        ],
    )?;
    app.manage(TrayState(Mutex::new(Some(TrayHandles {
        status: status.clone(),
        login: login.clone(),
    }))));

    TrayIconBuilder::with_id(TRAY_ID)
        .icon(tauri::include_image!("icons/tray-template@2x.png"))
        .icon_as_template(true)
        .tooltip("IYAGI Layout")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => crate::window::show_settings(app),
            "permission" => {
                if let Err(error) = layout_macos::permission::open_settings() {
                    tracing::warn!(%error, "cannot open accessibility settings");
                }
            }
            "login" => {
                let enabled = !layout_macos::login_item::status().is_enabled();
                match layout_macos::login_item::set_enabled(enabled) {
                    Ok(status) => tracing::info!(?status, "login item toggled"),
                    Err(error) => tracing::warn!(%error, "login item toggle failed"),
                }
                refresh_status(app);
            }
            "logs" => {
                if let Err(error) = crate::window::open_log_dir(app.clone()) {
                    tracing::warn!(%error, "cannot open the log directory");
                }
            }
            "quit" => {
                tracing::info!("quit requested from the tray");
                crate::hotkeys::unregister_all(app);
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}

/// 상태 줄과 로그인 체크를 현재 값으로 다시 그린다.
pub fn refresh_status(app: &AppHandle) {
    let Some(state) = app.try_state::<TrayState>() else {
        return;
    };
    let guard = state.0.lock().expect("tray lock");
    if let Some(h) = guard.as_ref() {
        let _ = h.status.set_text(status_text(app));
        let _ = h
            .login
            .set_checked(layout_macos::login_item::status().is_enabled());
    }
}

/// 상태 줄 우선순위: 권한 없음 → 설정 파일 오류 → 등록 실패 → 정상(단축키 요약).
fn status_text(app: &AppHandle) -> String {
    if !layout_macos::permission::is_trusted() {
        return crate::i18n::t(Msg::NeedsPermission).into();
    }
    let settings = app.state::<crate::settings::SettingsState>();
    if let Some(error) = settings.load_error() {
        return format!("{}: {error}", crate::i18n::t(Msg::SettingsFileError));
    }
    if let Some(hk) = app.try_state::<crate::hotkeys::HotkeyState>() {
        let failures = hk.failures();
        if !failures.is_empty() {
            let keys: Vec<&str> = failures.iter().map(|f| f.shortcut.as_str()).collect();
            return format!("{}: {}", crate::i18n::t(Msg::HotkeyFailed), keys.join(", "));
        }
    }
    let s = settings.current();
    let summary: Vec<String> = s
        .shortcuts
        .entries()
        .iter()
        .map(|(a, k)| format!("{} {}", pretty(k), crate::hotkeys::action_label(*a)))
        .collect();
    summary.join(" · ")
}

/// `Control+Super+ArrowLeft` → `⌃⌘←`.
pub fn pretty(spec: &str) -> String {
    spec.split('+')
        .map(|t| match t.trim().to_ascii_uppercase().as_str() {
            "CONTROL" | "CTRL" => "⌃".to_string(),
            "SUPER" | "COMMAND" | "CMD" | "COMMANDORCONTROL" | "CMDORCTRL" => "⌘".to_string(),
            "ALT" | "OPTION" => "⌥".to_string(),
            "SHIFT" => "⇧".to_string(),
            "ARROWLEFT" | "LEFT" => "←".to_string(),
            "ARROWRIGHT" | "RIGHT" => "→".to_string(),
            "ARROWUP" | "UP" => "↑".to_string(),
            "ARROWDOWN" | "DOWN" => "↓".to_string(),
            "ENTER" | "RETURN" => "↩".to_string(),
            "BACKSPACE" => "⌫".to_string(),
            other => other.strip_prefix("KEY").unwrap_or(other).to_string(),
        })
        .collect()
}
