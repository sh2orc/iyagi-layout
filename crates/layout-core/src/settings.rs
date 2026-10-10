//! 설정 파일 모델(`settings.json`). 파싱은 관대하게(모르는 키 무시, 빠진 키는 기본값),
//! 검증은 엄격하게(잘못된 값은 이유와 함께 거부) 한다. 로그인 항목 여부는 OS가
//! 진실의 원천이라 여기에 두지 않는다.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::geometry::Action;

pub const CURRENT_VERSION: u32 = 1;
pub const MAX_GAP: u32 = 32;
pub const MAX_CHAIN_WINDOW_MS: u32 = 10_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shortcuts {
    pub left: String,
    pub right: String,
    pub up: String,
    pub down: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            left: "Alt+Super+ArrowLeft".into(),
            right: "Alt+Super+ArrowRight".into(),
            up: "Alt+Super+ArrowUp".into(),
            down: "Alt+Super+ArrowDown".into(),
        }
    }
}

impl Shortcuts {
    pub fn get(&self, action: Action) -> &str {
        match action {
            Action::Left => &self.left,
            Action::Right => &self.right,
            Action::Up => &self.up,
            Action::Down => &self.down,
        }
    }

    pub fn entries(&self) -> [(Action, &str); 4] {
        [
            (Action::Left, &self.left),
            (Action::Right, &self.right),
            (Action::Up, &self.up),
            (Action::Down, &self.down),
        ]
    }
}

/// Chromium/Electron 앱의 `AXEnhancedUserInterface` 처리 정책(§4-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnhancedUiPolicy {
    /// 설정 전에 끄고, 보조 기술(VoiceOver 등)이 켜져 있으면 되돌린다.
    #[default]
    Automatic,
    /// 끄고 항상 되돌린다.
    DisableEnable,
    /// 끄고 되돌리지 않는다.
    DisableOnly,
    /// 건드리지 않는다.
    Off,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub version: u32,
    pub shortcuts: Shortcuts,
    pub gap: u32,
    pub cycle_thirds: bool,
    /// 축이 다른 방향키가 이 시간(ms) 안에 이어지면 현재 창을 다시 나눈다. 0이면 끔.
    pub chain_window_ms: u32,
    pub enhanced_ui_policy: EnhancedUiPolicy,
    pub log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            shortcuts: Shortcuts::default(),
            gap: 0,
            cycle_thirds: false,
            chain_window_ms: 3000,
            enhanced_ui_policy: EnhancedUiPolicy::Automatic,
            log_level: "info".into(),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SettingsError {
    #[error("settings.json is not valid JSON: {0}")]
    Json(String),
    #[error("unsupported settings version {0} (this build understands {CURRENT_VERSION})")]
    Version(u32),
    #[error("gap must be between 0 and {MAX_GAP}, got {0}")]
    Gap(u32),
    #[error("chain_window_ms must be between 0 and {MAX_CHAIN_WINDOW_MS}, got {0}")]
    ChainWindow(u32),
    #[error("shortcut for {action}: {reason}")]
    Shortcut {
        action: &'static str,
        reason: String,
    },
    #[error("two actions share the shortcut {0}")]
    DuplicateShortcut(String),
    #[error("log_level must be one of error, warn, info, debug, trace")]
    LogLevel,
}

const MODIFIERS: &[&str] = &[
    "CONTROL",
    "CTRL",
    "COMMAND",
    "CMD",
    "SUPER",
    "OPTION",
    "ALT",
    "SHIFT",
    "COMMANDORCONTROL",
    "COMMANDORCTRL",
    "CMDORCTRL",
    "CMDORCONTROL",
];

/// 단축키 문자열의 형태 검사(global-hotkey 파서의 규칙 중 OS 무관 부분):
/// `+`로 나눈 토큰이 수정키 1개 이상 + 키 1개, 수정키는 앞에, 대소문자 무시.
/// 키 이름의 실제 존재 여부는 앱 계층이 진짜 파서로 다시 확인한다.
pub fn check_shortcut_shape(spec: &str) -> Result<(), String> {
    let tokens: Vec<&str> = spec.split('+').map(str::trim).collect();
    if tokens.iter().any(|t| t.is_empty()) {
        return Err("empty token (check the '+' separators)".into());
    }
    if tokens.len() < 2 {
        return Err("needs at least one modifier and a key, e.g. Control+Super+ArrowLeft".into());
    }
    let (key, mods) = tokens.split_last().expect("len >= 2");
    for m in mods {
        if !MODIFIERS.contains(&m.to_ascii_uppercase().as_str()) {
            return Err(format!(
                "'{m}' is not a modifier (modifiers must come first)"
            ));
        }
    }
    if MODIFIERS.contains(&key.to_ascii_uppercase().as_str()) {
        return Err("the last token must be a key, not a modifier".into());
    }
    Ok(())
}

/// 비교용 정규화: 대소문자·공백 무시, 별칭 통일(Ctrl=Control, Cmd=Super, Option=Alt,
/// Left=ArrowLeft …), 수정키 순서 무시. 같은 조합을 다르게 적어도 중복으로 잡는다.
pub fn normalize_shortcut(spec: &str) -> String {
    fn canonical(token: &str) -> String {
        let upper = token.trim().to_ascii_uppercase();
        match upper.as_str() {
            "CTRL" => "CONTROL".into(),
            "CMD" | "COMMAND" => "SUPER".into(),
            "OPTION" => "ALT".into(),
            "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => "COMMANDORCONTROL".into(),
            "LEFT" => "ARROWLEFT".into(),
            "RIGHT" => "ARROWRIGHT".into(),
            "UP" => "ARROWUP".into(),
            "DOWN" => "ARROWDOWN".into(),
            "RETURN" => "ENTER".into(),
            "ESC" => "ESCAPE".into(),
            _ => upper,
        }
    }
    let tokens: Vec<String> = spec.split('+').map(canonical).collect();
    let Some((key, mods)) = tokens.split_last() else {
        return String::new();
    };
    let mut mods: Vec<&String> = mods.iter().collect();
    mods.sort();
    mods.into_iter()
        .map(String::as_str)
        .chain(std::iter::once(key.as_str()))
        .collect::<Vec<_>>()
        .join("+")
}

impl Settings {
    pub fn parse(json: &str) -> Result<Settings, SettingsError> {
        let settings: Settings =
            serde_json::from_str(json).map_err(|e| SettingsError::Json(e.to_string()))?;
        settings.validate()?;
        Ok(settings)
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("settings serialize") + "\n"
    }

    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.version != CURRENT_VERSION {
            return Err(SettingsError::Version(self.version));
        }
        if self.gap > MAX_GAP {
            return Err(SettingsError::Gap(self.gap));
        }
        if self.chain_window_ms > MAX_CHAIN_WINDOW_MS {
            return Err(SettingsError::ChainWindow(self.chain_window_ms));
        }
        let mut seen = Vec::new();
        for (action, spec) in self.shortcuts.entries() {
            check_shortcut_shape(spec).map_err(|reason| SettingsError::Shortcut {
                action: action.key(),
                reason,
            })?;
            let norm = normalize_shortcut(spec);
            if seen.contains(&norm) {
                return Err(SettingsError::DuplicateShortcut(spec.to_string()));
            }
            seen.push(norm);
        }
        if !["error", "warn", "info", "debug", "trace"].contains(&self.log_level.as_str()) {
            return Err(SettingsError::LogLevel);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_valid_and_round_trip() {
        let s = Settings::default();
        s.validate().unwrap();
        let back = Settings::parse(&s.to_json_pretty()).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn missing_and_unknown_keys_are_tolerated() {
        let s = Settings::parse(r#"{"gap": 8, "future_key": true}"#).unwrap();
        assert_eq!(s.gap, 8);
        assert_eq!(s.shortcuts, Shortcuts::default());
        assert_eq!(s.version, CURRENT_VERSION);
    }

    #[test]
    fn rejects_bad_values_with_reasons() {
        assert!(matches!(Settings::parse("{"), Err(SettingsError::Json(_))));
        assert_eq!(
            Settings::parse(r#"{"gap": 99}"#),
            Err(SettingsError::Gap(99))
        );
        assert_eq!(
            Settings::parse(r#"{"version": 7}"#),
            Err(SettingsError::Version(7))
        );
        assert_eq!(
            Settings::parse(r#"{"log_level": "loud"}"#),
            Err(SettingsError::LogLevel)
        );
        let dup = Settings::parse(
            r#"{"shortcuts": {"left": "alt+super+left", "right": "Option+Super+ArrowLeft"}}"#,
        );
        assert!(matches!(dup, Err(SettingsError::DuplicateShortcut(_))));
        let shape = Settings::parse(r#"{"shortcuts": {"up": "ArrowUp+Control"}}"#);
        assert!(matches!(
            shape,
            Err(SettingsError::Shortcut { action: "up", .. })
        ));
    }

    #[test]
    fn normalization_unifies_aliases_and_modifier_order() {
        assert_eq!(
            normalize_shortcut("ctrl+super+left"),
            normalize_shortcut("Super+Control+ArrowLeft")
        );
        assert_ne!(
            normalize_shortcut("Control+Super+ArrowLeft"),
            normalize_shortcut("Control+Alt+ArrowLeft")
        );
    }

    #[test]
    fn shortcut_shape_rules() {
        assert!(check_shortcut_shape("Control+Super+ArrowLeft").is_ok());
        assert!(check_shortcut_shape("cmd+shift+KeyC").is_ok());
        assert!(check_shortcut_shape("ArrowLeft").is_err());
        assert!(check_shortcut_shape("Control+").is_err());
        assert!(check_shortcut_shape("Control+Shift").is_err());
        assert!(check_shortcut_shape("Meta+ArrowLeft").is_err());
    }
}
