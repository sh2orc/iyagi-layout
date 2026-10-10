//! 설정의 단축키를 Carbon 전역 핫키로 등록한다(`layout_macos::hotkey`, 이벤트 탭 없음).
//! 설정이 바뀌면 전부 해제하고 다시 등록. 실패는 키별로 기록해 트레이·설정 창에 보여 준다.

use std::sync::Mutex;

use layout_core::{Action, Settings};
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct HotkeyState {
    failures: Mutex<Vec<HotkeyFailure>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HotkeyFailure {
    pub action: &'static str,
    pub shortcut: String,
    pub reason: String,
}

impl HotkeyState {
    pub fn failures(&self) -> Vec<HotkeyFailure> {
        self.failures.lock().expect("hotkey lock").clone()
    }
}

/// 실제 파서로 검증(저장 전에 설정 창이 부른다). 성공하면 정규화 표기.
pub fn validate(spec: &str) -> Result<String, String> {
    layout_macos::hotkey::parse(spec)
        .map(|p| p.canonical)
        .map_err(|e| e.to_string())
}

/// 메인 스레드에서 호출해야 한다(setup, 동기 커맨드, 트레이 이벤트는 모두 메인 스레드).
pub fn apply(app: &AppHandle, settings: &Settings) {
    layout_macos::hotkey::unregister_all();
    let mut failures = Vec::new();
    for (action, spec) in settings.shortcuts.entries() {
        let handle = app.clone();
        let result = layout_macos::hotkey::register(spec, move || {
            // Carbon 핸들러는 메인 스레드 — 바로 실행한다.
            crate::actions::run(&handle, action);
        });
        match result {
            Ok(parsed) => {
                tracing::info!(action = action.key(), shortcut = %parsed.canonical, "hotkey registered")
            }
            Err(error) => {
                tracing::warn!(action = action.key(), shortcut = spec, %error, "hotkey registration failed");
                failures.push(HotkeyFailure {
                    action: action.key(),
                    shortcut: spec.to_string(),
                    reason: error.to_string(),
                });
            }
        }
    }
    *app.state::<HotkeyState>()
        .failures
        .lock()
        .expect("hotkey lock") = failures;
}

pub fn unregister_all(_app: &AppHandle) {
    layout_macos::hotkey::unregister_all();
}

pub fn action_label(action: Action) -> &'static str {
    crate::i18n::t(layout_core::i18n::Msg::action(action))
}
