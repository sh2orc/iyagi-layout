//! 포커스 창 찾기와 프레임 읽기/쓰기(§4-2). 쓰기 순서는 크기 → 위치 → 크기(Rectangle).

use std::sync::Mutex;
use std::time::{Duration, Instant};

use layout_core::settings::EnhancedUiPolicy;
use layout_core::Rect;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};
use objc2_application_services::AXUIElement;
use objc2_core_foundation::{CFRetained, CGPoint, CGSize};

use crate::ax::{self, AxError};

/// 다른 앱의 창 하나. `app`은 그 창을 가진 앱 요소(Enhanced UI 처리용).
pub struct WindowHandle {
    element: CFRetained<AXUIElement>,
    app: CFRetained<AXUIElement>,
    pub pid: libc::pid_t,
    pub bundle_id: Option<String>,
}

/// 포커스 창: 포커스 앱의 AXFocusedWindow → 첫 AXWindows → 전역 포커스 요소의 AXWindow.
/// 포커스 앱은 시스템 전역 값을 먼저 보고, 그 값이 비면(크롬은 최전면인데도 비는 경우가 있다)
/// `frontmost_pid`로 만든 앱 요소로 넘어간다. 이 값을 인자로 받는 까닭은 NSWorkspace가
/// 메인 스레드 전용이라서 — 스냅 워커 스레드는 시작 시 메인 스레드가 모아 준 값을 쓴다.
/// 나머지 AX·NSRunningApplication 호출은 스레드 안전이라 워커에서 그대로 돈다.
pub fn focused_with_frontmost(
    frontmost_pid: Option<libc::pid_t>,
) -> Result<Option<WindowHandle>, AxError> {
    if !crate::permission::is_trusted() {
        return Err(AxError::NotTrusted);
    }
    // SAFETY: 인자 없는 생성.
    let system = unsafe { AXUIElement::new_system_wide() };
    let app = match ax::attr_element(&system, ax::FOCUSED_APPLICATION) {
        Ok(Some(app)) => app,
        _ => match frontmost_pid.map(|pid| {
            // SAFETY: 살아 있는 최전면 앱의 PID로 만든 요소다.
            unsafe { AXUIElement::new_application(pid) }
        }) {
            Some(app) => {
                tracing::debug!("no system-wide focused app — using the frontmost app");
                app
            }
            None => return Ok(None),
        },
    };
    let pid = ax::pid_of(&app)?;
    let element = match ax::attr_element(&app, ax::FOCUSED_WINDOW)? {
        Some(w) => Some(w),
        None => match ax::attr_elements(&app, ax::WINDOWS)?.into_iter().next() {
            Some(w) => Some(w),
            None => match ax::attr_element(&system, ax::FOCUSED_UI_ELEMENT)? {
                Some(focused) => {
                    if ax::attr_string(&focused, ax::ROLE)?.as_deref() == Some(ax::ROLE_WINDOW) {
                        Some(focused)
                    } else {
                        ax::attr_element(&focused, ax::WINDOW)?
                    }
                }
                None => None,
            },
        },
    };
    let Some(element) = element else {
        return Ok(None);
    };
    let bundle_id = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
        .and_then(|a| a.bundleIdentifier())
        .map(|s| s.to_string());
    Ok(Some(WindowHandle {
        element,
        app,
        pid,
        bundle_id,
    }))
}

/// 포커스 창 찾기(NSWorkspace 폴백 포함). 메인 스레드에서 부를 때 쓴다(개발용 probe).
pub fn focused() -> Result<Option<WindowHandle>, AxError> {
    focused_with_frontmost(frontmost_app_info().map(|(pid, _)| pid))
}

/// 최전면 앱의 PID와 번들 ID. NSWorkspace는 메인 스레드에서만 부른다 — 스냅 워커는
/// 시작 시 메인 스레드가 모아 준 이 값을 받아 쓴다. NSWorkspace의 값은 앱 런루프에서
/// 최신이라 키를 누른 순간의 앱이 나온다.
pub fn frontmost_app_info() -> Option<(libc::pid_t, Option<String>)> {
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let bundle = app.bundleIdentifier().map(|s| s.to_string());
    Some((app.processIdentifier(), bundle))
}

impl WindowHandle {
    pub fn frame(&self) -> Result<Rect, AxError> {
        let p = ax::attr_point(&self.element, ax::POSITION)?.ok_or(AxError::Type("position"))?;
        let s = ax::attr_size(&self.element, ax::SIZE)?.ok_or(AxError::Type("size"))?;
        Ok(Rect::new(p.x, p.y, s.width, s.height))
    }

    /// 크기 속성을 바꿀 수 있는가. 판정할 수 없으면 가능하다고 본다(Rectangle과 같다).
    pub fn is_resizable(&self) -> bool {
        ax::is_settable(&self.element, ax::SIZE).unwrap_or(true)
    }

    pub fn is_fullscreen(&self) -> bool {
        ax::attr_bool(&self.element, ax::FULL_SCREEN)
            .ok()
            .flatten()
            .unwrap_or(false)
    }

    pub fn enhanced_ui(&self) -> Option<bool> {
        ax::attr_bool(&self.app, ax::ENHANCED_USER_INTERFACE)
            .ok()
            .flatten()
    }

    /// 목표 프레임 적용 후 실제 프레임을 돌려준다. 크기 고정 창은 위치만 옮긴다.
    pub fn set_frame(&self, target: Rect, policy: EnhancedUiPolicy) -> Result<Rect, AxError> {
        let enhanced_was_on = match policy {
            EnhancedUiPolicy::Off => false,
            _ => self.enhanced_ui() == Some(true),
        };
        if enhanced_was_on {
            let _ = ax::set_bool(&self.app, ax::ENHANCED_USER_INTERFACE, false);
        }
        let result = self.apply(target);
        if enhanced_was_on {
            let restore = match policy {
                EnhancedUiPolicy::DisableEnable => true,
                EnhancedUiPolicy::Automatic => assistive_technology_active(),
                EnhancedUiPolicy::DisableOnly | EnhancedUiPolicy::Off => false,
            };
            if restore {
                let _ = ax::set_bool(&self.app, ax::ENHANCED_USER_INTERFACE, true);
            }
        }
        result
    }

    fn apply(&self, target: Rect) -> Result<Rect, AxError> {
        let size = CGSize::new(target.w, target.h);
        let pos = CGPoint::new(target.x, target.y);
        let resizable = self.is_resizable();
        if resizable {
            ax::set_size(&self.element, ax::SIZE, size)?;
        }
        ax::set_point(&self.element, ax::POSITION, pos)?;
        if resizable {
            ax::set_size(&self.element, ax::SIZE, size)?;
        }
        let mut actual = self.frame()?;
        // 최소 크기 등으로 밀렸으면 위치만 한 번 더 맞춘다(목표 영역 안에 머물게).
        if (actual.x - target.x).abs() > 1.0 || (actual.y - target.y).abs() > 1.0 {
            ax::set_point(&self.element, ax::POSITION, pos)?;
            actual = self.frame()?;
        }
        Ok(actual)
    }
}

/// VoiceOver 또는 스위치 제어가 켜져 있는가(Enhanced UI를 되돌릴지 판단).
/// 판단에 `defaults` 프로세스를 두 번 띄워야 해서(수십 ms) 60초 캐시한다 — 값은
/// 사용자가 시스템 설정을 바꿀 때만 달라진다. 기본 정책(Automatic)에서 Chromium 계열
/// 창을 옮길 때마다 불리니 캐시가 없으면 스냅 지연이 그만큼 커진다.
fn assistive_technology_active() -> bool {
    static CACHE: Mutex<Option<(Instant, bool)>> = Mutex::new(None);
    const TTL: Duration = Duration::from_secs(60);

    let read = |domain: &str, key: &str| {
        std::process::Command::new("defaults")
            .args(["read", domain, key])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "1")
            .unwrap_or(false)
    };
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((at, cached)) = *cache {
        if at.elapsed() < TTL {
            return cached;
        }
    }
    let active = read("com.apple.universalaccess", "voiceOverOnOffKey")
        || read("com.apple.universalaccess", "switchControlEnabled");
    *cache = Some((Instant::now(), active));
    active
}
