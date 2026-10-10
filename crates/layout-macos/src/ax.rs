//! AXUIElement 호출의 안전한 껍데기. 속성 이름은 C 헤더의 `#define` 문자열이라
//! 바인딩에 상수가 없다 — 여기서 문자열로 둔다.

use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_application_services::{AXError as RawAxError, AXUIElement, AXValue, AXValueType};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFRetained, CFString, CFType, CGPoint, CGSize, ConcreteType,
};
use thiserror::Error;

pub const FOCUSED_APPLICATION: &str = "AXFocusedApplication";
pub const FOCUSED_WINDOW: &str = "AXFocusedWindow";
pub const FOCUSED_UI_ELEMENT: &str = "AXFocusedUIElement";
pub const WINDOWS: &str = "AXWindows";
pub const WINDOW: &str = "AXWindow";
pub const POSITION: &str = "AXPosition";
pub const SIZE: &str = "AXSize";
pub const ROLE: &str = "AXRole";
pub const FULL_SCREEN: &str = "AXFullScreen";
pub const ENHANCED_USER_INTERFACE: &str = "AXEnhancedUserInterface";
pub const ROLE_WINDOW: &str = "AXWindow";

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AxError {
    #[error("accessibility permission not granted")]
    NotTrusted,
    #[error("accessibility call {call} failed: {code}")]
    Call { call: &'static str, code: i32 },
    #[error("attribute {0} has an unexpected type")]
    Type(&'static str),
    #[error("must run on the main thread")]
    NotMainThread,
    #[error("{0}")]
    Other(String),
}

pub fn check(call: &'static str, err: RawAxError) -> Result<(), AxError> {
    if err == RawAxError::Success {
        Ok(())
    } else if err == RawAxError::APIDisabled {
        Err(AxError::NotTrusted)
    } else {
        Err(AxError::Call { call, code: err.0 })
    }
}

pub fn cfstr(s: &str) -> CFRetained<CFString> {
    CFString::from_str(s)
}

/// 속성 값 복사. 속성이 없거나 값이 없으면 `Ok(None)`, 권한 문제는 `Err`.
pub fn copy_attr(el: &AXUIElement, name: &str) -> Result<Option<CFRetained<CFType>>, AxError> {
    let mut out: *const CFType = std::ptr::null();
    // SAFETY: `out`은 유효한 스택 변수이고, 성공 시 +1 참조가 들어온다.
    let err = unsafe { el.copy_attribute_value(&cfstr(name), NonNull::from(&mut out)) };
    match err {
        RawAxError::Success => {
            // SAFETY: Copy 규칙으로 소유권이 넘어온 포인터.
            Ok(NonNull::new(out as *mut CFType).map(|p| unsafe { CFRetained::from_raw(p) }))
        }
        RawAxError::NoValue | RawAxError::AttributeUnsupported => Ok(None),
        RawAxError::APIDisabled => Err(AxError::NotTrusted),
        other => Err(AxError::Call {
            call: "AXUIElementCopyAttributeValue",
            code: other.0,
        }),
    }
}

pub fn attr_as<T: ConcreteType + 'static>(
    el: &AXUIElement,
    name: &str,
    what: &'static str,
) -> Result<Option<CFRetained<T>>, AxError> {
    match copy_attr(el, name)? {
        None => Ok(None),
        Some(v) => v.downcast::<T>().map(Some).map_err(|_| AxError::Type(what)),
    }
}

pub fn attr_bool(el: &AXUIElement, name: &str) -> Result<Option<bool>, AxError> {
    Ok(attr_as::<CFBoolean>(el, name, "bool")?.map(|b| b.value()))
}

pub fn attr_string(el: &AXUIElement, name: &str) -> Result<Option<String>, AxError> {
    Ok(attr_as::<CFString>(el, name, "string")?.map(|s| s.to_string()))
}

pub fn attr_element(
    el: &AXUIElement,
    name: &str,
) -> Result<Option<CFRetained<AXUIElement>>, AxError> {
    attr_as::<AXUIElement>(el, name, "element")
}

pub fn attr_elements(
    el: &AXUIElement,
    name: &str,
) -> Result<Vec<CFRetained<AXUIElement>>, AxError> {
    let Some(array) = attr_as::<CFArray>(el, name, "array")? else {
        return Ok(Vec::new());
    };
    // SAFETY: AXWindows 배열의 원소는 CF 객체(AXUIElement)다 — CFType으로 보고 downcast한다.
    let array: &CFArray<CFType> = unsafe { array.cast_unchecked::<CFType>() };
    let mut out = Vec::new();
    for i in 0..array.len() {
        if let Some(item) = array.get(i) {
            if let Ok(e) = item.downcast::<AXUIElement>() {
                out.push(e);
            }
        }
    }
    Ok(out)
}

fn ax_value_get<T: Copy>(
    value: &AXValue,
    ty: AXValueType,
    what: &'static str,
) -> Result<T, AxError> {
    let mut out = std::mem::MaybeUninit::<T>::uninit();
    // SAFETY: `out`은 `ty`가 뜻하는 구조체 크기만큼의 유효한 메모리.
    let ok = unsafe { value.value(ty, NonNull::new_unchecked(out.as_mut_ptr() as *mut c_void)) };
    if ok {
        // SAFETY: AXValueGetValue가 true를 돌려주면 값이 채워져 있다.
        Ok(unsafe { out.assume_init() })
    } else {
        Err(AxError::Type(what))
    }
}

pub fn attr_point(el: &AXUIElement, name: &str) -> Result<Option<CGPoint>, AxError> {
    attr_as::<AXValue>(el, name, "point")?
        .map(|v| ax_value_get::<CGPoint>(&v, AXValueType::CGPoint, "point"))
        .transpose()
}

pub fn attr_size(el: &AXUIElement, name: &str) -> Result<Option<CGSize>, AxError> {
    attr_as::<AXValue>(el, name, "size")?
        .map(|v| ax_value_get::<CGSize>(&v, AXValueType::CGSize, "size"))
        .transpose()
}

fn ax_value_new<T>(ty: AXValueType, mut value: T) -> Result<CFRetained<AXValue>, AxError> {
    // SAFETY: `value`는 `ty`에 맞는 구조체이며 호출 동안 살아 있다(AXValueCreate가 복사).
    unsafe {
        AXValue::new(
            ty,
            NonNull::new_unchecked(&mut value as *mut T as *mut c_void),
        )
    }
    .ok_or_else(|| AxError::Other("AXValueCreate returned NULL".into()))
}

pub fn set_point(el: &AXUIElement, name: &str, p: CGPoint) -> Result<(), AxError> {
    let v = ax_value_new(AXValueType::CGPoint, p)?;
    // SAFETY: 유효한 요소·이름·값.
    check("AXUIElementSetAttributeValue(position)", unsafe {
        el.set_attribute_value(&cfstr(name), &v)
    })
}

pub fn set_size(el: &AXUIElement, name: &str, s: CGSize) -> Result<(), AxError> {
    let v = ax_value_new(AXValueType::CGSize, s)?;
    // SAFETY: 유효한 요소·이름·값.
    check("AXUIElementSetAttributeValue(size)", unsafe {
        el.set_attribute_value(&cfstr(name), &v)
    })
}

pub fn set_bool(el: &AXUIElement, name: &str, b: bool) -> Result<(), AxError> {
    // SAFETY: 유효한 요소·이름·값.
    check("AXUIElementSetAttributeValue(bool)", unsafe {
        el.set_attribute_value(&cfstr(name), CFBoolean::new(b))
    })
}

pub fn is_settable(el: &AXUIElement, name: &str) -> Result<bool, AxError> {
    let mut settable: u8 = 0;
    // SAFETY: 유효한 out 포인터.
    check("AXUIElementIsAttributeSettable", unsafe {
        el.is_attribute_settable(&cfstr(name), NonNull::from(&mut settable))
    })?;
    Ok(settable != 0)
}

pub fn pid_of(el: &AXUIElement) -> Result<libc::pid_t, AxError> {
    let mut pid: libc::pid_t = 0;
    // SAFETY: 유효한 out 포인터.
    check("AXUIElementGetPid", unsafe {
        el.pid(NonNull::from(&mut pid))
    })?;
    Ok(pid)
}

/// 이 프로세스가 보내는 모든 AX 메시지의 상한(초). 응답 없는 앱이 우리를 오래 막지 않게.
pub fn set_global_timeout(seconds: f32) {
    // SAFETY: 시스템 전역 요소에 설정하면 프로세스 전역 타임아웃이 된다.
    unsafe {
        let sys = AXUIElement::new_system_wide();
        let _ = sys.set_messaging_timeout(seconds);
    }
}
