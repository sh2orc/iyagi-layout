//! 로그인 항목(`SMAppService.mainApp`, macOS 13+). OS 상태가 진실의 원천이다.

use objc2_service_management::{SMAppService, SMAppServiceStatus};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginItemStatus {
    /// 등록돼 있고 로그인 시 실행된다.
    Enabled,
    /// 등록했지만 사용자가 시스템 설정에서 꺼 둔 상태.
    RequiresApproval,
    NotRegistered,
    NotFound,
}

impl LoginItemStatus {
    pub fn is_enabled(self) -> bool {
        self == LoginItemStatus::Enabled
    }
}

/// 진단용: OS가 돌려준 원시 상태 값.
pub fn raw_status() -> isize {
    // SAFETY: 상태 읽기만 한다.
    unsafe { SMAppService::mainAppService().status().0 }
}

pub fn status() -> LoginItemStatus {
    // SAFETY: 메인 앱 서비스 조회는 인자가 없고 상태 읽기만 한다.
    let raw = unsafe { SMAppService::mainAppService().status() };
    match raw {
        SMAppServiceStatus::Enabled => LoginItemStatus::Enabled,
        SMAppServiceStatus::RequiresApproval => LoginItemStatus::RequiresApproval,
        SMAppServiceStatus::NotRegistered => LoginItemStatus::NotRegistered,
        _ => LoginItemStatus::NotFound,
    }
}

pub fn set_enabled(enabled: bool) -> Result<LoginItemStatus, String> {
    // SAFETY: 메인 앱 서비스에 대한 등록/해제. 실패는 NSError로 돌아온다.
    let result = unsafe {
        let service = SMAppService::mainAppService();
        if enabled {
            service.registerAndReturnError()
        } else {
            service.unregisterAndReturnError()
        }
    };
    result.map_err(|e| e.localizedDescription().to_string())?;
    Ok(status())
}

/// 시스템 설정 › 일반 › 로그인 항목 열기.
pub fn open_settings() {
    // SAFETY: 인자 없는 시스템 설정 열기.
    unsafe { SMAppService::openSystemSettingsLoginItems() }
}
