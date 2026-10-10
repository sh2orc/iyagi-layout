//! 번들 안 실행 파일을 터미널에서 직접 부를 때의 진단 플래그. GUI를 띄우지 않고 끝난다.
//!   "IYAGI Layout.app/Contents/MacOS/iyagi-layout-app" --login-item status|on|off
//!   … --permission

/// 처리했으면 `true`(바로 종료), 아니면 `false`(정상 기동).
pub fn handle() -> bool {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--login-item") => {
            let out = match args.get(2).map(String::as_str) {
                Some("on") => layout_macos::login_item::set_enabled(true).map(|s| format!("{s:?}")),
                Some("off") => {
                    layout_macos::login_item::set_enabled(false).map(|s| format!("{s:?}"))
                }
                _ => Ok(format!(
                    "{:?} (raw {})",
                    layout_macos::login_item::status(),
                    layout_macos::login_item::raw_status()
                )),
            };
            match out {
                Ok(s) => println!("login item: {s}"),
                Err(e) => {
                    eprintln!("login item error: {e}");
                    std::process::exit(1);
                }
            }
            true
        }
        Some("--lang") => {
            use layout_core::i18n::Msg;
            println!(
                "lang: {} (system: {:?})",
                crate::i18n::current().code(),
                layout_macos::locale::preferred_languages()
            );
            println!(
                "{} | {} | {}",
                crate::i18n::t(Msg::Settings),
                crate::i18n::t(Msg::LaunchAtLogin),
                crate::i18n::t(Msg::Quit)
            );
            true
        }
        Some("--permission") => {
            println!(
                "accessibility trusted: {}",
                layout_macos::permission::is_trusted()
            );
            true
        }
        _ => false,
    }
}
