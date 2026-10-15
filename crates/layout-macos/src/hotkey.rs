//! 전역 단축키 — Carbon `RegisterEventHotKey`만 쓴다. `CGEventTap`은 설치하지 않는다
//! (전역 이벤트 탭은 프로세스가 느려지면 시스템 전체 키 입력을 지연시킨다).
//! 핸들러는 NSApplication 런루프가 Carbon 이벤트를 돌릴 때 **메인 스레드**에서 불린다.

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

use thiserror::Error;

type OSStatus = i32;
type EventRef = *mut c_void;
type EventHandlerCallRef = *mut c_void;
type EventHandlerRef = *mut c_void;
type EventHotKeyRef = *mut c_void;
type EventTargetRef = *mut c_void;
type EventHandlerUPP = extern "C" fn(EventHandlerCallRef, EventRef, *mut c_void) -> OSStatus;

#[repr(C)]
struct EventTypeSpec {
    event_class: u32,
    event_kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct EventHotKeyID {
    signature: u32,
    id: u32,
}

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn GetEventDispatcherTarget() -> EventTargetRef;
    fn InstallEventHandler(
        target: EventTargetRef,
        handler: EventHandlerUPP,
        num_types: u32,
        list: *const EventTypeSpec,
        user_data: *mut c_void,
        out_ref: *mut EventHandlerRef,
    ) -> OSStatus;
    fn RegisterEventHotKey(
        key_code: u32,
        modifiers: u32,
        hot_key_id: EventHotKeyID,
        target: EventTargetRef,
        options: u32,
        out_ref: *mut EventHotKeyRef,
    ) -> OSStatus;
    fn UnregisterEventHotKey(hot_key: EventHotKeyRef) -> OSStatus;
    fn GetEventParameter(
        event: EventRef,
        name: u32,
        wanted_type: u32,
        actual_type: *mut u32,
        buffer_size: u32,
        actual_size: *mut u32,
        out_data: *mut c_void,
    ) -> OSStatus;
}

const fn fourcc(s: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*s)
}
const K_EVENT_CLASS_KEYBOARD: u32 = fourcc(b"keyb");
const K_EVENT_HOT_KEY_PRESSED: u32 = 5;
const K_EVENT_PARAM_DIRECT_OBJECT: u32 = fourcc(b"----");
const TYPE_EVENT_HOT_KEY_ID: u32 = fourcc(b"hkid");
const SIGNATURE: u32 = fourcc(b"IYLT");

const CMD_KEY: u32 = 1 << 8;
const SHIFT_KEY: u32 = 1 << 9;
const OPTION_KEY: u32 = 1 << 11;
const CONTROL_KEY: u32 = 1 << 12;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HotkeyError {
    #[error("{0}")]
    Parse(String),
    #[error("RegisterEventHotKey failed (OSStatus {0}) — another app may own this combination")]
    Register(OSStatus),
    #[error("InstallEventHandler failed (OSStatus {0})")]
    Install(OSStatus),
    #[error("must be called on the main thread")]
    NotMainThread,
}

/// 파싱 결과: Carbon 수정키 비트, 가상 키코드, 정규화 표기.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedHotkey {
    pub modifiers: u32,
    pub key_code: u32,
    pub canonical: String,
}

/// `Control+Super+ArrowLeft` 같은 문자열을 파싱한다(global-hotkey와 같은 이름 규칙).
pub fn parse(spec: &str) -> Result<ParsedHotkey, HotkeyError> {
    let tokens: Vec<&str> = spec.split('+').map(str::trim).collect();
    if tokens.len() < 2 || tokens.iter().any(|t| t.is_empty()) {
        return Err(HotkeyError::Parse(
            "needs at least one modifier and a key, e.g. Control+Super+ArrowLeft".into(),
        ));
    }
    let (key, mods) = tokens.split_last().expect("len >= 2");
    let mut modifiers = 0u32;
    let mut names = Vec::new();
    for m in mods {
        let (bit, name) = match m.to_ascii_uppercase().as_str() {
            "CONTROL" | "CTRL" => (CONTROL_KEY, "Control"),
            "SUPER" | "COMMAND" | "CMD" | "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL"
            | "CMDORCONTROL" => (CMD_KEY, "Super"),
            "ALT" | "OPTION" => (OPTION_KEY, "Alt"),
            "SHIFT" => (SHIFT_KEY, "Shift"),
            other => return Err(HotkeyError::Parse(format!("'{other}' is not a modifier"))),
        };
        if modifiers & bit == 0 {
            names.push(name);
        }
        modifiers |= bit;
    }
    let (key_code, key_name) =
        key_code(key).ok_or_else(|| HotkeyError::Parse(format!("unknown key '{key}'")))?;
    let order = ["Control", "Alt", "Shift", "Super"];
    names.sort_by_key(|n| order.iter().position(|o| o == n));
    let canonical = format!("{}+{}", names.join("+"), key_name);
    Ok(ParsedHotkey {
        modifiers,
        key_code,
        canonical,
    })
}

/// W3C `code` 이름(및 흔한 별칭) → macOS 가상 키코드(ANSI 배열).
fn key_code(name: &str) -> Option<(u32, String)> {
    let upper = name.to_ascii_uppercase();
    let (code, canonical): (u32, &str) = match upper.as_str() {
        "ARROWLEFT" | "LEFT" => (0x7B, "ArrowLeft"),
        "ARROWRIGHT" | "RIGHT" => (0x7C, "ArrowRight"),
        "ARROWDOWN" | "DOWN" => (0x7D, "ArrowDown"),
        "ARROWUP" | "UP" => (0x7E, "ArrowUp"),
        "ENTER" | "RETURN" => (0x24, "Enter"),
        "TAB" => (0x30, "Tab"),
        "SPACE" => (0x31, "Space"),
        "BACKSPACE" => (0x33, "Backspace"),
        "ESCAPE" | "ESC" => (0x35, "Escape"),
        "DELETE" => (0x75, "Delete"),
        "HOME" => (0x73, "Home"),
        "END" => (0x77, "End"),
        "PAGEUP" => (0x74, "PageUp"),
        "PAGEDOWN" => (0x79, "PageDown"),
        "MINUS" => (0x1B, "Minus"),
        "EQUAL" => (0x18, "Equal"),
        "BRACKETLEFT" => (0x21, "BracketLeft"),
        "BRACKETRIGHT" => (0x1E, "BracketRight"),
        "SEMICOLON" => (0x29, "Semicolon"),
        "QUOTE" => (0x27, "Quote"),
        "BACKSLASH" => (0x2A, "Backslash"),
        "COMMA" => (0x2B, "Comma"),
        "PERIOD" => (0x2F, "Period"),
        "SLASH" => (0x2C, "Slash"),
        "BACKQUOTE" => (0x32, "Backquote"),
        "F1" => (0x7A, "F1"),
        "F2" => (0x78, "F2"),
        "F3" => (0x63, "F3"),
        "F4" => (0x76, "F4"),
        "F5" => (0x60, "F5"),
        "F6" => (0x61, "F6"),
        "F7" => (0x62, "F7"),
        "F8" => (0x64, "F8"),
        "F9" => (0x65, "F9"),
        "F10" => (0x6D, "F10"),
        "F11" => (0x67, "F11"),
        "F12" => (0x6F, "F12"),
        _ => {
            let letter = upper
                .strip_prefix("KEY")
                .filter(|l| l.len() == 1)
                .map(|l| l.as_bytes()[0]);
            let digit = upper
                .strip_prefix("DIGIT")
                .filter(|d| d.len() == 1)
                .map(|d| d.as_bytes()[0]);
            if let Some(l) = letter.filter(|l| l.is_ascii_uppercase()) {
                let code = match l {
                    b'A' => 0x00,
                    b'S' => 0x01,
                    b'D' => 0x02,
                    b'F' => 0x03,
                    b'H' => 0x04,
                    b'G' => 0x05,
                    b'Z' => 0x06,
                    b'X' => 0x07,
                    b'C' => 0x08,
                    b'V' => 0x09,
                    b'B' => 0x0B,
                    b'Q' => 0x0C,
                    b'W' => 0x0D,
                    b'E' => 0x0E,
                    b'R' => 0x0F,
                    b'Y' => 0x10,
                    b'T' => 0x11,
                    b'O' => 0x1F,
                    b'U' => 0x20,
                    b'I' => 0x22,
                    b'P' => 0x23,
                    b'L' => 0x25,
                    b'J' => 0x26,
                    b'K' => 0x28,
                    b'N' => 0x2D,
                    b'M' => 0x2E,
                    _ => return None,
                };
                return Some((code, format!("Key{}", l as char)));
            }
            if let Some(d) = digit.filter(|d| d.is_ascii_digit()) {
                let code = match d {
                    b'1' => 0x12,
                    b'2' => 0x13,
                    b'3' => 0x14,
                    b'4' => 0x15,
                    b'5' => 0x17,
                    b'6' => 0x16,
                    b'7' => 0x1A,
                    b'8' => 0x1C,
                    b'9' => 0x19,
                    b'0' => 0x1D,
                    _ => return None,
                };
                return Some((code, format!("Digit{}", d as char)));
            }
            return None;
        }
    };
    Some((code, canonical.to_string()))
}

type Callback = Box<dyn Fn() + Send + 'static>;

struct Registry {
    callbacks: HashMap<u32, Callback>,
    refs: Vec<SendPtr>,
    next_id: u32,
}

struct SendPtr(EventHotKeyRef);
// SAFETY: Carbon 핫키 참조는 메인 스레드에서만 만들고 해제한다. 보관만 다른 스레드에서 가능.
unsafe impl Send for SendPtr {}

static REGISTRY: Mutex<Option<Registry>> = Mutex::new(None);
static HANDLER_INSTALLED: OnceLock<Result<(), HotkeyError>> = OnceLock::new();

extern "C" fn handler(_call: EventHandlerCallRef, event: EventRef, _user: *mut c_void) -> OSStatus {
    let mut hot_key_id = EventHotKeyID {
        signature: 0,
        id: 0,
    };
    // SAFETY: Carbon이 넘긴 이벤트에서 hkid 파라미터를 같은 크기의 구조체로 읽는다.
    let status = unsafe {
        GetEventParameter(
            event,
            K_EVENT_PARAM_DIRECT_OBJECT,
            TYPE_EVENT_HOT_KEY_ID,
            std::ptr::null_mut(),
            std::mem::size_of::<EventHotKeyID>() as u32,
            std::ptr::null_mut(),
            &mut hot_key_id as *mut EventHotKeyID as *mut c_void,
        )
    };
    tracing::debug!(
        status,
        signature = hot_key_id.signature,
        id = hot_key_id.id,
        "carbon hotkey event"
    );
    if status != 0 || hot_key_id.signature != SIGNATURE {
        return 0;
    }
    // 콜백을 꺼내는 동안만 잠근다 — 콜백 안에서 재등록해도 교착하지 않게 복제하지 않고
    // 잠금 밖에서 호출할 수 없으므로, 콜백은 짧게 끝나야 한다.
    let guard = REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(cb) = guard.as_ref().and_then(|r| r.callbacks.get(&hot_key_id.id)) {
        invoke_guarded(cb);
    }
    0
}

/// 콜백을 호출하되 Rust panic과 Objective-C 예외가 C 경계(Carbon 핸들러)를 넘지 못하게 막는다.
/// 넘으면 "panic in a function that cannot unwind"로 프로세스가 abort된다.
fn invoke_guarded(cb: &Callback) {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        objc2::exception::catch(AssertUnwindSafe(cb))
    }));
    match outcome {
        Ok(Ok(())) => {}
        Ok(Err(exception)) => tracing::error!(
            ?exception,
            "hotkey callback raised an Objective-C exception"
        ),
        Err(_) => tracing::error!("hotkey callback panicked"),
    }
}

fn install_handler() -> Result<(), HotkeyError> {
    HANDLER_INSTALLED
        .get_or_init(|| {
            let spec = EventTypeSpec {
                event_class: K_EVENT_CLASS_KEYBOARD,
                event_kind: K_EVENT_HOT_KEY_PRESSED,
            };
            let mut out: EventHandlerRef = std::ptr::null_mut();
            // SAFETY: 이벤트 디스패처 타깃에 정적 콜백을 한 번 설치한다(프로세스 수명).
            // Tauri(tao)의 NSApplication 루프에서는 응용 프로그램 타깃으로 핫키 이벤트가
            // 오지 않는 것이 실기에서 확인됐다 — MASShortcut과 같이 디스패처 타깃을 쓴다.
            let status = unsafe {
                InstallEventHandler(
                    GetEventDispatcherTarget(),
                    handler,
                    1,
                    &spec,
                    std::ptr::null_mut(),
                    &mut out,
                )
            };
            tracing::debug!(status, "carbon hotkey handler installed");
            if status == 0 {
                Ok(())
            } else {
                Err(HotkeyError::Install(status))
            }
        })
        .clone()
}

fn ensure_main_thread() -> Result<(), HotkeyError> {
    objc2::MainThreadMarker::new()
        .map(|_| ())
        .ok_or(HotkeyError::NotMainThread)
}

/// 단축키 등록. 같은 조합을 다른 앱이 이미 잡았으면 `Register` 오류.
pub fn register(
    spec: &str,
    callback: impl Fn() + Send + 'static,
) -> Result<ParsedHotkey, HotkeyError> {
    ensure_main_thread()?;
    install_handler()?;
    let parsed = parse(spec)?;
    let mut guard = REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let registry = guard.get_or_insert_with(|| Registry {
        callbacks: HashMap::new(),
        refs: Vec::new(),
        next_id: 1,
    });
    let id = registry.next_id;
    registry.next_id += 1;
    let mut out: EventHotKeyRef = std::ptr::null_mut();
    // SAFETY: 유효한 키코드·수정키·타깃. 실패하면 out은 NULL.
    let status = unsafe {
        RegisterEventHotKey(
            parsed.key_code,
            parsed.modifiers,
            EventHotKeyID {
                signature: SIGNATURE,
                id,
            },
            GetEventDispatcherTarget(),
            0,
            &mut out,
        )
    };
    if status != 0 {
        return Err(HotkeyError::Register(status));
    }
    registry.refs.push(SendPtr(out));
    registry.callbacks.insert(id, Box::new(callback));
    Ok(parsed)
}

/// 등록한 모든 단축키 해제.
pub fn unregister_all() {
    if ensure_main_thread().is_err() {
        return;
    }
    let mut guard = REGISTRY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(registry) = guard.as_mut() {
        for r in registry.refs.drain(..) {
            // SAFETY: register가 돌려준 유효한 참조를 한 번만 해제한다.
            unsafe {
                UnregisterEventHotKey(r.0);
            }
        }
        registry.callbacks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_bindings() {
        let p = parse("Control+Super+ArrowLeft").unwrap();
        assert_eq!(p.modifiers, CONTROL_KEY | CMD_KEY);
        assert_eq!(p.key_code, 0x7B);
        assert_eq!(p.canonical, "Control+Super+ArrowLeft");
        assert_eq!(
            parse("cmd+ctrl+left").unwrap().canonical,
            "Control+Super+ArrowLeft"
        );
        assert_eq!(parse("Alt+Shift+KeyC").unwrap().key_code, 0x08);
        assert_eq!(parse("Super+Digit1").unwrap().key_code, 0x12);
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse("ArrowLeft").is_err());
        assert!(parse("Meta+ArrowLeft").is_err());
        assert!(parse("Control+Nope").is_err());
        assert!(parse("Control+Shift").is_err());
    }
}
