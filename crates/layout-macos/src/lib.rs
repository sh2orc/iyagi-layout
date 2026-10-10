//! macOS 전용 래퍼. 모든 `unsafe` FFI 호출은 이 crate 안에서 끝나고, 밖으로는 안전한
//! Rust 타입만 나간다. 좌표는 `layout_core::Rect`(CG 좌표)로 통일한다.
//!
//! 모듈: `permission`(손쉬운 사용 TCC), `screens`(NSScreen → CG 좌표),
//! `window`(AXUIElement 읽기/쓰기), `login_item`(SMAppService).

#![cfg(target_os = "macos")]

pub mod hotkey;
pub mod launch;
pub mod locale;
pub mod login_item;
pub mod permission;
pub mod screens;
pub mod window;

mod ax;

pub use ax::AxError;

/// 프로세스 전역 AX 메시지 타임아웃(0.5초). 기동 시 한 번 부른다.
pub fn ax_timeout() {
    ax::set_global_timeout(0.5);
}
