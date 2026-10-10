//! 이번 실행이 로그인 직후 자동 실행(로그인 항목)인지 추정한다.
//!
//! SMAppService가 띄운 앱은 `kAEOpenApplication` 이벤트에 로그인 항목 표식을 담아 주지만
//! 그 이벤트는 처리 중에만 읽을 수 있어 Tauri setup 시점에는 접근할 수 없다. 대신
//! 사용자 세션의 `loginwindow` 프로세스 경과 시간을 본다 — 로그인 뒤 `LOGIN_GRACE` 안에
//! 떴으면 자동 실행으로 본다. 어림 규칙이라 틀려도 설정 창이 하나 더 뜨거나 안 뜨는 정도다.

use std::process::Command;
use std::time::Duration;

pub const LOGIN_GRACE: Duration = Duration::from_secs(120);

/// 세션 로그인 뒤 경과 시간. 알 수 없으면 `None`.
pub fn session_age() -> Option<Duration> {
    // SAFETY: getuid는 실패하지 않는다.
    let uid = unsafe { libc::getuid() };
    let pid = Command::new("pgrep")
        .args(["-u", &uid.to_string(), "-x", "loginwindow"])
        .output()
        .ok()?;
    let pid = String::from_utf8_lossy(&pid.stdout)
        .lines()
        .next()?
        .trim()
        .to_string();
    let etimes = Command::new("ps")
        .args(["-o", "etimes=", "-p", &pid])
        .output()
        .ok()?;
    let secs: u64 = String::from_utf8_lossy(&etimes.stdout)
        .trim()
        .parse()
        .ok()?;
    Some(Duration::from_secs(secs))
}

pub fn launched_at_login() -> bool {
    session_age().is_some_and(|age| age <= LOGIN_GRACE)
}
