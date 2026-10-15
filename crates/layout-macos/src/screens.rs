//! NSScreen → `layout_core::Screen`(CG 좌표). 메인 스레드에서만 부른다.

use layout_core::{cocoa_to_cg, Rect, Screen};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{msg_send, MainThreadMarker};
use objc2_app_kit::NSScreen;
use objc2_foundation::ns_string;

use crate::AxError;

fn rect(r: objc2_foundation::NSRect) -> Rect {
    Rect::new(r.origin.x, r.origin.y, r.size.width, r.size.height)
}

/// 디스플레이 ID. `NSScreen.CGDirectDisplayID`는 최신 macOS에만 있어(macOS 14.2에서는
/// unrecognized selector 예외) 오래된 `deviceDescription["NSScreenNumber"]`를 읽는다.
fn display_id(screen: &NSScreen) -> u32 {
    let description = screen.deviceDescription();
    let number: Option<Retained<AnyObject>> = description.objectForKey(ns_string!("NSScreenNumber"));
    // SAFETY: NSScreenNumber 값은 NSNumber이고 unsignedIntValue에 응답한다.
    number.map_or(0, |n| unsafe { msg_send![&*n, unsignedIntValue] })
}

/// 연결된 화면 목록. 첫 번째가 주 화면(메뉴 막대, 원점 (0,0)).
pub fn list() -> Result<Vec<Screen>, AxError> {
    let mtm = MainThreadMarker::new().ok_or(AxError::NotMainThread)?;
    let screens = NSScreen::screens(mtm);
    let Some(primary) = screens.iter().next() else {
        return Ok(Vec::new());
    };
    let primary_height = primary.frame().size.height;
    Ok(screens
        .iter()
        .enumerate()
        .map(|(i, s)| Screen {
            id: display_id(&s),
            frame: cocoa_to_cg(primary_height, rect(s.frame())),
            visible: cocoa_to_cg(primary_height, rect(s.visibleFrame())),
            is_primary: i == 0,
            scale: s.backingScaleFactor(),
        })
        .collect())
}
