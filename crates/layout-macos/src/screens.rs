//! NSScreen → `layout_core::Screen`(CG 좌표). 메인 스레드에서만 부른다.

use layout_core::{cocoa_to_cg, Rect, Screen};
use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;

use crate::AxError;

fn rect(r: objc2_foundation::NSRect) -> Rect {
    Rect::new(r.origin.x, r.origin.y, r.size.width, r.size.height)
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
            id: s.CGDirectDisplayID(),
            frame: cocoa_to_cg(primary_height, rect(s.frame())),
            visible: cocoa_to_cg(primary_height, rect(s.visibleFrame())),
            is_primary: i == 0,
            scale: s.backingScaleFactor(),
        })
        .collect())
}
