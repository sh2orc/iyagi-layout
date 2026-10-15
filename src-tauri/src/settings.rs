//! settings.json 로드·검증·저장. 손상된 파일은 덮어쓰지 않고 기본값으로 동작하며
//! 그 사실(`load_error`)을 트레이·설정 창에 드러낸다.

use std::path::PathBuf;
use std::sync::Mutex;

use layout_core::Settings;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

pub const SETTINGS_CHANGED_EVENT: &str = "iyagi-layout://settings-changed";

#[derive(Default)]
pub struct SettingsState {
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    settings: Settings,
    path: Option<PathBuf>,
    load_error: Option<String>,
}

impl SettingsState {
    pub fn current(&self) -> Settings {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .settings
            .clone()
    }

    pub fn load_error(&self) -> Option<String> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .load_error
            .clone()
    }

    pub fn path(&self) -> Option<PathBuf> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .path
            .clone()
    }
}

/// 설정 창에 보내는 묶음: 설정값 + 파일 경로 + 로드 오류.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub settings: Settings,
    pub path: Option<String>,
    pub load_error: Option<String>,
}

fn settings_file(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .app_config_dir()
        .ok()
        .map(|d| d.join("settings.json"))
}

/// 기동 시 한 번: 파일이 없으면 기본값을 쓰고, 있으면 읽어 검증한다.
pub fn load_into_state(app: &AppHandle) {
    let state = app.state::<SettingsState>();
    let mut inner = state
        .inner
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(path) = settings_file(app) else {
        inner.load_error = Some("config directory unavailable".into());
        return;
    };
    inner.path = Some(path.clone());
    match std::fs::read_to_string(&path) {
        Ok(text) => match Settings::parse(&text) {
            Ok(settings) => {
                inner.settings = settings;
                inner.load_error = None;
            }
            Err(e) => {
                inner.settings = Settings::default();
                inner.load_error = Some(e.to_string());
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            inner.settings = Settings::default();
            inner.load_error = write_file(&path, &inner.settings).err();
        }
        Err(e) => {
            inner.settings = Settings::default();
            inner.load_error = Some(format!("cannot read {}: {e}", path.display()));
        }
    }
}

fn write_file(path: &PathBuf, settings: &Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    }
    // 임시 파일에 쓴 뒤 교체 — 저장 중 종료돼도 반쪽 파일이 남지 않는다.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, settings.to_json_pretty())
        .and_then(|_| std::fs::rename(&tmp, path))
        .map_err(|e| format!("cannot write {}: {e}", path.display()))
}

/// 검증 → 저장 → 상태 교체 → 알림. 실패하면 아무것도 바뀌지 않는다.
pub fn apply(app: &AppHandle, settings: Settings) -> Result<Settings, String> {
    settings.validate().map_err(|e| e.to_string())?;
    let state = app.state::<SettingsState>();
    {
        let mut inner = state
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(path) = inner.path.clone() {
            write_file(&path, &settings)?;
        }
        inner.settings = settings.clone();
        inner.load_error = None;
    }
    crate::hotkeys::apply(app, &settings);
    crate::tray::refresh_status(app);
    let _ = app.emit(SETTINGS_CHANGED_EVENT, &settings);
    tracing::info!("settings saved");
    Ok(settings)
}

pub fn view(app: &AppHandle) -> SettingsView {
    let state = app.state::<SettingsState>();
    SettingsView {
        settings: state.current(),
        path: state.path().map(|p| p.display().to_string()),
        load_error: state.load_error(),
    }
}

#[tauri::command]
pub fn settings_get(app: AppHandle) -> SettingsView {
    view(&app)
}

#[tauri::command]
pub fn settings_set(app: AppHandle, settings: Settings) -> Result<SettingsView, String> {
    apply(&app, settings)?;
    Ok(view(&app))
}

#[tauri::command]
pub fn settings_path(app: AppHandle) -> Option<String> {
    app.state::<SettingsState>()
        .path()
        .map(|p| p.display().to_string())
}
