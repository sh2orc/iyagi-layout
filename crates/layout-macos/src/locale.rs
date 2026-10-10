//! 시스템 선호 언어(시스템 설정 › 일반 › 언어 및 지역의 우선순위 목록).

use objc2_foundation::NSLocale;

/// `["ko-KR", "en-US"]` 같은 BCP 47 태그, 우선순위 순.
pub fn preferred_languages() -> Vec<String> {
    NSLocale::preferredLanguages()
        .iter()
        .map(|s| s.to_string())
        .collect()
}
