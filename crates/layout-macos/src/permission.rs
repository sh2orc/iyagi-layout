//! 손쉬운 사용(Accessibility) TCC 권한.

use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};

pub const SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

/// 지금 신뢰된 클라이언트인가(안내창 없음).
pub fn is_trusted() -> bool {
    // SAFETY: 인자 없는 순수 조회.
    unsafe { AXIsProcessTrusted() }
}

/// 신뢰되지 않았으면 시스템 안내창을 띄운다(비동기). 반환값은 현재 상태.
pub fn prompt() -> bool {
    // SAFETY: 정적 키와 정적 CFBoolean으로 만든 사전은 호출 동안 유효하다.
    unsafe {
        let key: &CFString = kAXTrustedCheckOptionPrompt;
        let dict =
            CFDictionary::<CFString, CFBoolean>::from_slices(&[key], &[CFBoolean::new(true)]);
        AXIsProcessTrustedWithOptions(Some(dict.as_opaque()))
    }
}

/// 시스템 설정 › 개인정보 보호 및 보안 › 손쉬운 사용 열기.
pub fn open_settings() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg(SETTINGS_URL)
        .spawn()
        .map(|_| ())
}
