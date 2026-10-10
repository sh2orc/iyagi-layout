//! OS에 의존하지 않는 순수 로직. 좌표는 모두 **CoreGraphics 전역 좌표**(주
//! 화면 왼쪽 위 원점, y 아래 방향, 단위 pt)로 통일한다 — Accessibility API가
//! 쓰는 좌표계다. Cocoa(NSScreen) 좌표는 `cocoa_to_cg`로 들어올 때 한 번만 바꾼다.

pub mod cycle;
pub mod geometry;
pub mod i18n;
pub mod settings;

pub use cycle::{CycleState, Fraction, Plan, Rules};
pub use geometry::{
    cocoa_to_cg, compute_chained, compute_target, pick_screen, Action, Rect, Screen,
};
pub use settings::{Settings, Shortcuts};
