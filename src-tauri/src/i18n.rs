//! 앱 언어. 시스템 선호 언어에서 한 번 정해 프로세스 동안 유지한다(언어를 바꾸면 앱을
//! 다시 실행해야 메뉴 막대 메뉴에 반영된다). `IYAGI_LANG=ko|en|ja|zh`로 강제할 수 있다(시험용).

use std::sync::OnceLock;

use layout_core::i18n::{tr, Lang, Msg};

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn current() -> Lang {
    *LANG.get_or_init(|| {
        if let Some(forced) = std::env::var("IYAGI_LANG")
            .ok()
            .as_deref()
            .and_then(Lang::from_tag)
        {
            return forced;
        }
        Lang::detect(layout_macos::locale::preferred_languages())
    })
}

pub fn t(msg: Msg) -> &'static str {
    tr(current(), msg)
}
