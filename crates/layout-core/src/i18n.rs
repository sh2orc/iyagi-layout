//! 지원 언어와 Rust 쪽 문구(메뉴 막대 메뉴·상태 줄). 설정 창(React) 문구는 `src/i18n.ts`.
//!
//! 규칙: 시스템 선호 언어 목록에서 **지원하는 첫 언어**를 쓰고(macOS 앱의 표준 동작),
//! 하나도 없으면 영어. 중국어는 간체(zh-Hans)로 통일한다 — 번체 지역(zh-TW, zh-HK)도 간체로 표시.

use serde::Serialize;

use crate::geometry::Action;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    En,
    Ko,
    Ja,
    Zh,
}

impl Lang {
    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Ko => "ko",
            Lang::Ja => "ja",
            Lang::Zh => "zh",
        }
    }

    /// `ko-KR`, `ja_JP`, `zh-Hans-CN` 같은 태그 → 지원 언어. 지원하지 않으면 `None`.
    pub fn from_tag(tag: &str) -> Option<Lang> {
        let primary = tag.trim().split(['-', '_']).next()?.to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Lang::En),
            "ko" => Some(Lang::Ko),
            "ja" => Some(Lang::Ja),
            "zh" => Some(Lang::Zh),
            _ => None,
        }
    }

    /// 선호 언어 목록(우선순위 순)에서 지원하는 첫 언어. 없으면 영어.
    pub fn detect<I, S>(tags: I) -> Lang
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        tags.into_iter()
            .find_map(|t| Lang::from_tag(t.as_ref()))
            .unwrap_or(Lang::En)
    }
}

/// 정적 문구.
#[derive(Debug, Clone, Copy)]
pub enum Msg {
    Settings,
    AccessibilitySettings,
    LaunchAtLogin,
    OpenLogs,
    Quit,
    NeedsPermission,
    /// 뒤에 `: 상세`를 붙인다.
    SettingsFileError,
    /// 뒤에 `: 키 목록`을 붙인다.
    HotkeyFailed,
    ActionLeft,
    ActionRight,
    ActionUp,
    ActionDown,
    /// 전체 화면 창은 건드리지 않는다(macOS가 전체 화면 Space에 묶어 둔다).
    FullScreenSkipped,
    /// 재시도 끝에도 포커스 창을 찾지 못했다.
    NoFocusedWindow,
    /// 지금 앞에 있는 창이 IYAGI Layout 자신의 창이다.
    OwnWindow,
    /// 창 위치·크기를 읽거나 쓰지 못했다.
    MoveFailed,
}

impl Msg {
    pub fn action(action: Action) -> Msg {
        match action {
            Action::Left => Msg::ActionLeft,
            Action::Right => Msg::ActionRight,
            Action::Up => Msg::ActionUp,
            Action::Down => Msg::ActionDown,
        }
    }
}

pub fn tr(lang: Lang, msg: Msg) -> &'static str {
    use Lang::*;
    use Msg::*;
    match (msg, lang) {
        (Settings, En) => "Settings…",
        (Settings, Ko) => "설정…",
        (Settings, Ja) => "設定…",
        (Settings, Zh) => "设置…",

        (AccessibilitySettings, En) => "Open Accessibility Settings",
        (AccessibilitySettings, Ko) => "손쉬운 사용 설정 열기",
        (AccessibilitySettings, Ja) => "アクセシビリティ設定を開く",
        (AccessibilitySettings, Zh) => "打开辅助功能设置",

        (LaunchAtLogin, En) => "Launch at Login",
        (LaunchAtLogin, Ko) => "로그인 시 실행",
        (LaunchAtLogin, Ja) => "ログイン時に起動",
        (LaunchAtLogin, Zh) => "登录时启动",

        (OpenLogs, En) => "Open Logs Folder",
        (OpenLogs, Ko) => "로그 폴더 열기",
        (OpenLogs, Ja) => "ログフォルダを開く",
        (OpenLogs, Zh) => "打开日志文件夹",

        (Quit, En) => "Quit",
        (Quit, Ko) => "종료",
        (Quit, Ja) => "終了",
        (Quit, Zh) => "退出",

        (NeedsPermission, En) => "⚠︎ Accessibility permission needed",
        (NeedsPermission, Ko) => "⚠︎ 손쉬운 사용 권한 필요",
        (NeedsPermission, Ja) => "⚠︎ アクセシビリティの許可が必要です",
        (NeedsPermission, Zh) => "⚠︎ 需要辅助功能权限",

        (SettingsFileError, En) => "⚠︎ Settings file error",
        (SettingsFileError, Ko) => "⚠︎ 설정 파일 오류",
        (SettingsFileError, Ja) => "⚠︎ 設定ファイルのエラー",
        (SettingsFileError, Zh) => "⚠︎ 设置文件错误",

        (HotkeyFailed, En) => "⚠︎ Shortcut registration failed",
        (HotkeyFailed, Ko) => "⚠︎ 단축키 등록 실패",
        (HotkeyFailed, Ja) => "⚠︎ ショートカットの登録に失敗",
        (HotkeyFailed, Zh) => "⚠︎ 快捷键注册失败",

        (ActionLeft, En) => "Left half",
        (ActionLeft, Ko) => "왼쪽 절반",
        (ActionLeft, Ja) => "左半分",
        (ActionLeft, Zh) => "左半屏",

        (ActionRight, En) => "Right half",
        (ActionRight, Ko) => "오른쪽 절반",
        (ActionRight, Ja) => "右半分",
        (ActionRight, Zh) => "右半屏",

        (ActionUp, En) => "Top half",
        (ActionUp, Ko) => "위 절반",
        (ActionUp, Ja) => "上半分",
        (ActionUp, Zh) => "上半屏",

        (ActionDown, En) => "Bottom half",
        (ActionDown, Ko) => "아래 절반",
        (ActionDown, Ja) => "下半分",
        (ActionDown, Zh) => "下半屏",

        (FullScreenSkipped, En) => "Full-screen windows can't be moved",
        (FullScreenSkipped, Ko) => "전체 화면 창은 옮길 수 없습니다",
        (FullScreenSkipped, Ja) => "全画面ウインドウは移動できません",
        (FullScreenSkipped, Zh) => "无法移动全屏窗口",

        (NoFocusedWindow, En) => "No focused window",
        (NoFocusedWindow, Ko) => "포커스된 창이 없습니다",
        (NoFocusedWindow, Ja) => "フォーカス中のウインドウがありません",
        (NoFocusedWindow, Zh) => "没有聚焦的窗口",

        (OwnWindow, En) => "Select another app's window first",
        (OwnWindow, Ko) => "다른 앱의 창을 먼저 선택하세요",
        (OwnWindow, Ja) => "先に他のアプリのウインドウを選択してください",
        (OwnWindow, Zh) => "请先选择其他应用的窗口",

        (MoveFailed, En) => "Could not move the window",
        (MoveFailed, Ko) => "창을 옮기지 못했습니다",
        (MoveFailed, Ja) => "ウインドウを移動できませんでした",
        (MoveFailed, Zh) => "无法移动窗口",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_map_to_supported_languages() {
        assert_eq!(Lang::from_tag("ko-KR"), Some(Lang::Ko));
        assert_eq!(Lang::from_tag("ko"), Some(Lang::Ko));
        assert_eq!(Lang::from_tag("ja_JP"), Some(Lang::Ja));
        assert_eq!(Lang::from_tag("zh-Hans-CN"), Some(Lang::Zh));
        assert_eq!(Lang::from_tag("zh-Hant-TW"), Some(Lang::Zh));
        assert_eq!(Lang::from_tag("en-GB"), Some(Lang::En));
        assert_eq!(Lang::from_tag("fr-FR"), None);
        assert_eq!(Lang::from_tag(""), None);
    }

    #[test]
    fn first_supported_preferred_language_wins_else_english() {
        assert_eq!(Lang::detect(["fr-FR", "ja-JP", "ko-KR"]), Lang::Ja);
        assert_eq!(Lang::detect(["de-DE", "es-ES"]), Lang::En);
        assert_eq!(Lang::detect(Vec::<String>::new()), Lang::En);
        assert_eq!(Lang::detect(["ko-KR", "en-US"]), Lang::Ko);
    }

    #[test]
    fn every_message_is_translated_and_non_empty() {
        let msgs = [
            Msg::Settings,
            Msg::AccessibilitySettings,
            Msg::LaunchAtLogin,
            Msg::OpenLogs,
            Msg::Quit,
            Msg::NeedsPermission,
            Msg::SettingsFileError,
            Msg::HotkeyFailed,
            Msg::ActionLeft,
            Msg::ActionRight,
            Msg::ActionUp,
            Msg::ActionDown,
            Msg::FullScreenSkipped,
            Msg::NoFocusedWindow,
            Msg::OwnWindow,
            Msg::MoveFailed,
        ];
        for lang in [Lang::En, Lang::Ko, Lang::Ja, Lang::Zh] {
            for m in msgs {
                assert!(!tr(lang, m).is_empty());
            }
        }
    }
}
