# IYAGI Layout 구현 방안

작성일: 2026-10-09 · 상태: **v1.0 — 결정 확정(2026-10-09), 구현 진행 중**. 티켓 진행 상황은 §10.

형제 프로젝트: [iyagi-term](../iyagi-term) (IYAGI Terminal). 이 문서는 iyagi-term의 구조·스크립트·문서 관례를 기준으로 IYAGI Layout의 구현 방안을 정한다. 아직 코드는 없다.

## 0. 결정이 필요한 항목 (토의 포인트)

2026-10-09 토의로 모두 확정했다. D3만 추천과 다르게 설정 창을 MVP에 포함한다.

| # | 항목 | 후보 | 추천 | 근거 요약 | 결정 |
|---|---|---|---|---|---|
| D1 | 기술 스택 | (a) Tauri 2 + Rust 워크스페이스 (b) Swift/AppKit 네이티브 (c) Rust 전용(tao + tray-icon + global-hotkey, 번들러 별도) | **(a)** | iyagi-term과 툴체인·서명·CI·문서 관례를 그대로 공유. 필요한 macOS API는 모두 Rust 바인딩이 있고 분량이 작다(§3, §4) |(a) |
| D2 | 로그인 시 자동 실행 | (a) `SMAppService.mainApp` 직접 호출 (b) tauri-plugin-autostart LaunchAgent (c) tauri-plugin-autostart AppleScript | **(a)** | 설정 앱 "로그인 시 열기"에 앱 이름으로 표시, 번들 ID 기준이라 앱을 옮겨도 유지, 상태 역조회 가능(§4-5) |(a) |
| D3 | MVP UI 범위 | (a) 트레이(메뉴 막대) 메뉴만 (b) 설정 창(React) 포함 | (a) | 추천은 (a)였으나 사용자가 (b)를 택함. 창은 지연 생성이라 유휴 메모리 영향은 없다 |**(b) 설정 창 포함** |
| D4 | 같은 방향키 반복 입력 | (a) 항상 1/2 (b) 1/2 → 2/3 → 1/3 순환 | **(a) 기본, (b)는 설정 옵션(기본 off)** | 요구사항은 "반으로". 순환은 Rectangle 사용자에게 익숙하지만 예측 가능성이 떨어진다 |(a)+옵션 |
| D5 | 멀티 모니터 | (a) 창이 있는 모니터 안에서만 (b) 이미 그 절반에 있으면 옆 모니터로 이동 | **(a)** MVP, (b)는 후속 옵션 | (b)는 창 상태 추적이 필요하고 오동작 시 창이 사라진 것처럼 보인다 |(a) |
| D6 | Tauri 버전 | (a) 최신 안정 2.12.x (b) iyagi-term과 동일 2.11.5 | **(a)** | 현재 플러그인(autostart 2.7.0)이 tauri ^2.12를 요구. Tauri 3은 alpha라 제외 |(a) |
| D7 | 설정 저장 | (a) JSON 파일 (b) SQLite(term-storage 방식) (c) tauri-plugin-store | **(a)** | 설정 항목 10개 미만, 사람이 직접 고칠 수 있어야 한다 |(a) |
| D8 | 저장소·공유 방식 | (a) 별도 저장소, 스크립트는 복사 (b) iyagi-term 모노레포에 편입 (c) 공용 빌드 스크립트 패키지 | **(a)** | 릴리스 주기가 다르고 코드 의존이 없다. 공용화는 세 번째 형제가 생길 때 |(a) |

이 외 소소한 확정 사항(이견 없으면 그대로 간다): 제품명 **IYAGI Layout**, 번들 식별자 `ai.iyagi.layout`, 실행 파일 `iyagi-layout`, 라이선스 Apache-2.0, 최소 macOS 13(Ventura), Apple Silicon 우선(Intel은 요청 시 cross build).

## 1. 목표와 범위

### 1-1. 한 줄 정의

메뉴 막대에 상주하며 **Ctrl+Cmd+방향키**로 현재 포커스된(다른 앱의) 창을 그 화면의 왼쪽·오른쪽·위·아래 **절반**에 맞춰 배치하는 macOS 유틸리티. 창을 닫아도 종료되지 않고, 로그인하면 자동으로 뜬다.

### 1-2. MVP (MUST)

1. `Ctrl+Cmd+←/→/↑/↓` → 포커스 창을 왼쪽/오른쪽/위/아래 절반으로 이동·크기 조절. 어느 앱의 창이든 대상이다.
2. 메뉴 막대 아이콘만 있고 Dock 아이콘·앱 메뉴가 없다(`LSUIElement`).
3. 앱 창(있다면)을 닫아도 프로세스는 남는다. 종료는 트레이 메뉴 "종료"로만.
4. 로그인 항목 등록/해제를 트레이 메뉴에서 토글하고, 실제 OS 상태와 일치하게 표시한다.
5. 손쉬운 사용(Accessibility) 권한이 없으면 안내하고 설정 창을 열어 준다. 권한이 없어도 앱은 죽지 않는다.
6. 단축키·간격·순환 옵션을 설정 창(React)에서 바꿀 수 있고, 설정 파일로도 고칠 수 있다.
7. 두 번 실행해도 인스턴스는 하나다(로그인 항목 + 수동 실행 중복 방지).
8. 개발 중 재빌드해도 손쉬운 사용 권한이 유지된다(안정 서명, §4-6).

### 1-3. 후속 (SHOULD / MAY)

- 최대화(`Ctrl+Cmd+Return`), 이전 크기 복원(`Ctrl+Cmd+Delete`), 중앙 정렬, 1/4 배치, 옆 모니터로 이동(D5-b), 1/3·2/3 순환(D4-b).
- 창 사이 간격(gap)과 화면 가장자리 여백의 분리.

### 1-4. 범위 밖 (MUST NOT)

- 자동 타일링 윈도 매니저(yabai류), 창 목록/스위처, 마우스 드래그 스냅.
- Mac App Store 배포(Accessibility API는 샌드박스에서 동작하지 않는다).
- Windows/Linux 지원. 이 프로젝트는 macOS 전용이며, iyagi-term과 달리 크로스플랫폼 추상화를 두지 않는다.

## 2. 왜 macOS 기본 타일링으로 끝내지 않는가

macOS 15(Sequoia)부터 창 타일링이 내장돼 있다: `Fn+Ctrl+←/→/↑/↓` 절반, `Fn+Ctrl+F` 채우기, `Fn+Ctrl+C` 중앙, `Fn+Ctrl+R` 되돌리기(Apple 안내 문서, §12). 그럼에도 별도 앱을 만드는 이유:

| 항목 | 기본 타일링 | IYAGI Layout |
|---|---|---|
| 단축키 | `Fn+Ctrl+방향키` 고정(앱 단축키로 메뉴 항목 이름을 다시 다는 우회만 가능) | `Ctrl+Cmd+방향키`, 설정 파일로 변경 |
| 여백 | "타일 창 여백" on/off만 | 간격 값 지정 |
| 반복 입력·복원·모니터 이동 | 없음/제한 | 후속 범위에서 추가 |
| 이 Mac의 현재 설정 | 가장자리 드래그 타일링·여백 모두 꺼져 있음(`com.apple.WindowManager`) | — |

두 기능은 공존한다. 기본 타일링(`Fn+Ctrl`)과 이 앱(`Ctrl+Cmd`)의 단축키는 충돌하지 않는다.

## 3. 기술 스택 비교 (D1)

| 기준 | (a) Tauri 2 + Rust | (b) Swift/AppKit | (c) Rust 전용 |
|---|---|---|---|
| 전역 단축키 | `tauri-plugin-global-shortcut` → `global-hotkey` → Carbon `RegisterEventHotKey` | Carbon `RegisterEventHotKey` 또는 `NSEvent` 모니터 | `global-hotkey` 직접 |
| 창 조작(AX) | `objc2-application-services`(AXUIElement/AXValue 바인딩 확인) | 네이티브 | 동일 바인딩 |
| 화면 기하 | `objc2-app-kit` `NSScreen` | 네이티브 | 동일 |
| 로그인 항목 | `objc2-service-management` `SMAppService` | 네이티브 | 동일 |
| 트레이·메뉴 | Tauri 내장(`tray-icon`, `muda`) | `NSStatusItem` | `tray-icon`+`muda` 직접, 이벤트 루프(`tao`) 직접 |
| 설정 UI | 트레이 메뉴 → 필요 시 WebView 창 | SwiftUI | 메뉴만(창은 직접 AppKit) |
| 번들·DMG·서명·공증 | Tauri 번들러 + iyagi-term 스크립트 재사용 | Xcode/`swift build` + 수동 번들 조립 + 같은 `codesign`/`notarytool` | `cargo-bundle` 등 별도 도구 |
| 유휴 메모리 | 창이 없으면 WebView 미생성 → 수십 MB 수준(실측 필요) | 가장 작음 | 작음 |
| iyagi-term과의 일관성 | 동일 | 다름(새 툴체인) | 절반 |
| 유지보수 | 사용자 주력 스택(Rust/Tauri/TS) | Swift 경험 필요 | Rust만, 보일러플레이트 많음 |
| 리스크 | Carbon API 의존(§11), AX 바인딩이 `unsafe` 호출 | 가장 낮음 | 이벤트 루프·번들러 직접 관리 |

추천은 (a)다. 근거:

1. 이 앱의 macOS 전용 코드는 AX 호출 10여 개, NSScreen 읽기, SMAppService 메서드 4개로 작다. 전부 검증된 Rust 바인딩이 있다(§4).
2. 서명·TCC 안정화(§4-6), DMG/공증, CI, 문서 관례를 iyagi-term에서 그대로 가져온다. 형제 프로젝트의 가장 큰 이점이다.
3. 핵심 로직을 Tauri에 의존하지 않는 crate(`layout-core`, `layout-macos`)로 분리하면, 훗날 셸만 Swift로 바꾸는 선택지도 남는다.

(b)를 택해야 할 신호: Tauri의 이벤트 루프/트레이가 macOS 26에서 문제를 일으키거나, 설정 UI를 네이티브로 만들고 싶을 때. 현재는 해당 없음.

## 4. macOS 메커니즘 검토 (검증 결과)

이 절의 버전·API 이름은 2026-10-09에 crates.io/docs.rs/이 Mac에서 확인한 값이다.

### 4-1. 전역 단축키

- **Carbon `RegisterEventHotKey`를 직접 호출한다**(`layout-macos::hotkey`, FFI 80줄). 권한이 필요 없고, 등록된 조합은 시스템이 먼저 소비해 최전면 앱에 전달되지 않는다. 핸들러는 NSApplication 런루프가 Carbon 이벤트를 돌릴 때 메인 스레드에서 불린다.
- 핸들러 설치와 핫키 등록의 타깃은 **`GetEventDispatcherTarget()`**이어야 한다(MASShortcut과 같다). `GetApplicationEventTarget()`으로는 Tauri(tao)의 NSApplication 루프에서 핸들러가 한 번도 불리지 않는 것을 실기에서 확인했다(2026-10-09, 합성 키 입력으로 재현). 합성 입력 검증은 HID 탭에 post한 CGEvent로는 핫키가 발화하지 않고, 세션 탭 post 또는 System Events `key code`로는 발화한다.
- **`tauri-plugin-global-shortcut`은 쓰지 않는다(2026-10-09 변경).** 그 플러그인의 `global-hotkey` 크레이트는 미디어 키용 `CGEventTap`(active filter)을 무조건 설치하는데, 실기에서 Tab·Backspace·한/영 전환이 시스템 전체에서 멈추는 현상이 났다. 전역 이벤트 탭은 설치한 프로세스가 조금만 느려져도 WindowServer가 키 전달을 기다리므로, 창 배치 앱에는 탭 자체를 두지 않는 것이 맞다.
- 단축키 문자열 규칙은 global-hotkey와 같다: 수정키 `Control|Ctrl`, `Command|Cmd|Super`, `Option|Alt`, `Shift`; 키는 W3C `code` 이름(`ArrowLeft|Left`, `KeyC`, `Digit1`, `Enter`, `F1`…). 저장 형식은 `Control+Super+ArrowLeft`로 정규화한다.
- 등록 실패(다른 앱이 같은 조합을 먼저 잡음)는 로그 + 트레이 상태 줄 + 설정 창에 표시하고 나머지 키는 계속 등록한다.
- 충돌 점검:
  - 시스템 기본 단축키 중 `Ctrl+Cmd+방향키`는 없다(기본 Spaces 전환은 `Ctrl+←/→`, 기본 타일링은 `Fn+Ctrl+방향키`).
  - **Xcode**는 `Ctrl+Cmd+←/→`를 "Go Back/Forward"로 쓴다. 전역 핫키가 이를 가리므로 설정 창에서 바꿀 수 있다.
  - 이 Mac에는 **Magnet**이 떠 있다(기본 `Ctrl+Opt+방향키`). 같은 조합을 쓰면 먼저 등록한 쪽이 이긴다.
  - QA 중 발견(2026-10-09): 손쉬운 사용 목록에서 **Magnet의 허용을 끄면** Tab·Backspace·한/영 전환이 시스템 전체에서 멈추고 키가 반복된다. Magnet이 권한 없이 이벤트 탭을 유지하며 생기는 Magnet 쪽 현상으로, 이 앱과 무관하다(앱을 완전히 종료해도 재현됨). 테스트할 때는 Magnet을 허용한 채 두거나 Magnet 자체를 종료한다.

### 4-2. 다른 앱의 창 조작 — Accessibility API

- crate: `objc2-application-services` **0.3.2** (features `HIServices`, `AXUIElement`, `AXValue`, `AXError`, `libc`). 확인된 API: `AXUIElement::new_system_wide()`, `new_application(pid)`, `copy_attribute_value`, `set_attribute_value`, `is_attribute_settable`, `AXValue::new(AXValueType, ptr)` / `AXValue::value(...)`(CGPoint/CGSize 포장·해제), `AXIsProcessTrustedWithOptions(Option<&CFDictionary>)`, `kAXTrustedCheckOptionPrompt`. 모두 `unsafe` 호출이라 `layout-macos` 안에서 안전한 Rust API로 감싼다.
- 대상 창 찾기(Rectangle의 순서): 시스템 전역 요소의 `AXFocusedApplication` → 그 앱의 `AXFocusedWindow` → 없으면 앱의 첫 `AXWindows` 항목 → 그래도 없으면 전역 `AXFocusedUIElement`가 속한 창.
- 프레임 읽기: `AXPosition`(CGPoint), `AXSize`(CGSize). 좌표계는 **주 화면 왼쪽 위가 원점, y 아래 방향, 단위 pt**(CoreGraphics 전역 좌표).
- 프레임 쓰기 순서: **크기 → 위치 → 크기**(Rectangle `setFrame`). AX는 크기와 위치를 따로 적용하고 macOS가 화면 밖 크기를 거부하므로 두 번째 크기 설정이 필요하다. 쓴 뒤 다시 읽어 최소 크기 등으로 결과가 다르면 목표 영역 안에 머물도록 위치만 한 번 더 보정한다.
- 권한: `AXIsProcessTrustedWithOptions({kAXTrustedCheckOptionPrompt: true})` 한 번으로 시스템 안내창을 띄운다. 거부/미설정이면 트레이 메뉴 "손쉬운 사용 설정 열기"가 `x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility`를 연다. 안내창은 실행당 최대 1회.
- 샌드박스 금지: App Sandbox가 켜지면 AX가 동작하지 않는다. Hardened Runtime은 무관(켠다). 별도 entitlement는 없다 — TCC 권한이다.
- 응답 없는 대상 앱 보호: `AXUIElementSetMessagingTimeout`(바인딩 존재 여부는 L03에서 확인)으로 요소당 0.5초 상한. MVP는 메인 스레드에서 AX를 호출하되, 멈춤이 체감되면 전용 워커 스레드로 옮긴다(후속 강화).

알려진 함정과 대응:

| 상황 | 증상 | 대응 |
|---|---|---|
| Chromium/Electron 앱(Chrome, VS Code, Slack 등)의 `AXEnhancedUserInterface`가 켜짐 | 이동이 애니메이션되거나 잘못된 위치 | 앱 요소의 `AXEnhancedUserInterface`를 설정 전에 끄고, VoiceOver 등 보조 기술이 켜져 있으면 되돌린다(Rectangle `automatic` 정책) |
| 크기 고정 창(시스템 설정, 계산기) | `AXSize` 설정 불가 | `is_attribute_settable(AXSize)`가 false면 위치만 옮겨 목표 영역 안에 둔다 |
| 전체 화면 창 | 이동 시 Space가 깨짐 | `AXFullScreen`이 true면 무시하고 알림(§6-5) |
| 창이 없는 포커스(Finder 데스크톱) | 대상 없음 | 짧게 재시도한 뒤 알림(§6-5) |
| JetBrains IDE 등 비표준 창 | 좌표 보고가 늦음 | 쓴 뒤 다시 읽어 보정. 재현되면 개별 우회 추가 |

### 4-3. 화면 기하와 좌표 변환

- `objc2-app-kit` `NSScreen::screens()`, `frame()`, `visibleFrame()`. `visibleFrame`은 메뉴 막대(노치 포함)와 Dock을 뺀 영역이다.
- Cocoa 좌표(왼쪽 아래 원점, y 위 방향) → AX/CG 좌표 변환: `y_cg = primary.frame.height - (y_cocoa + height)`. `primary`는 `screens[0]`(메뉴 막대가 있는 주 화면, 원점 (0,0)). 이 변환은 `layout-core`의 순수 함수로 두고 단위 테스트한다.
- 대상 화면 선택: 창 프레임과 교집합 면적이 가장 큰 화면 → 없으면 창 중심점을 포함하는 화면 → 마우스 커서 화면 → 주 화면.
- 캐시 금지: 키 입력마다 화면 목록을 새로 읽는다(모니터 연결/해상도 변경 대응).
- 결과 좌표는 정수 pt로 반올림한다(0.5pt 경계의 흐림 방지).

### 4-4. 백그라운드 상주와 창 생명주기

- `src-tauri/Info.plist`에 `LSUIElement = true`. Tauri는 설정 파일과 같은 디렉터리의 `Info.plist`를 번들 Info.plist에 자동 병합한다(설정 스키마 확인). 보강으로 `setup`에서 `app.set_activation_policy(ActivationPolicy::Accessory)`.
- `tauri.conf.json`의 `app.windows`를 비운다(스키마상 선택 항목). 시작 시 창도 WebView도 없다. `build.frontendDist`도 선택 항목이라 MVP에는 프런트엔드 디렉터리가 없어도 된다(L01에서 실제 `tauri build` 통과를 확인한다).
- 마지막 창이 닫혀도 종료하지 않기: Tauri 문서의 트레이 패턴 그대로 `RunEvent::ExitRequested { api, code: None, .. }`에서 `api.prevent_exit()`. 트레이 "종료"는 `app.exit(0)` → `code: Some(0)` → 통과. (후속 설정 창은 `CloseRequested`에서 `prevent_close` + `hide`.)
- Accessory 정책이라 앱 메뉴가 없고 `Cmd+Q`가 앱에 닿지 않는다. 종료 경로는 트레이뿐이다. 종료 시 단축키를 명시적으로 해제한다(`unregister_all`).
- 단일 인스턴스: `tauri-plugin-single-instance` **2.5.2**. 두 번째 실행은 첫 인스턴스에 알리고 바로 종료한다.
- 로그인 시 실행되면 창 없이 트레이 아이콘만 뜬다. Dock에 나타나지 않는다.

### 4-5. 로그인 자동 실행 (D2)

| 방식 | 사용자에게 보이는 곳 | 식별 기준 | 추가 권한 | 상태 역조회 | 비고 |
|---|---|---|---|---|---|
| **SMAppService.mainApp** (macOS 13+) | 설정 › 일반 › 로그인 항목 › "로그인 시 열기"에 앱 이름·아이콘 | 번들 ID | 없음 | `status()`: `Enabled` / `RequiresApproval` / `NotRegistered` / `NotFound` | Apple 권장 API. 첫 등록 때 "백그라운드 항목 추가됨" 알림 |
| tauri-plugin-autostart `LaunchAgent`(플러그인 기본) | "백그라운드에서 허용" 목록 | `~/Library/LaunchAgents/ai.iyagi.layout.plist`의 **절대 경로** | 없음 | plist 존재 여부뿐 | 앱을 옮기면 깨짐 |
| tauri-plugin-autostart `AppleScript` | "로그인 시 열기" | System Events 로그인 항목 | **자동화(Automation) 권한** 추가 요청 | AppleScript 질의 | 권한 창이 하나 더 뜸 |

추천은 SMAppService다. `objc2-service-management` **0.3.2**(기본 feature에 `SMAppService` 포함): `SMAppService::mainAppService()`, `registerAndReturnError()`, `unregisterAndReturnError()`, `status()`. 네 호출 모두 `unsafe`라 `layout-macos::login_item` 모듈로 감싼다. 트레이의 "로그인 시 실행" 체크 상태는 설정 파일이 아니라 **매번 `status()`로 읽는다**(사용자가 설정 앱에서 끄면 `RequiresApproval`로 바뀌므로 그 사실을 메뉴에 표시한다).

실기 확인(2026-10-09, macOS 26.6): 한 번도 등록한 적 없는 앱은 `status()`가 `NotRegistered`가 아니라 **`NotFound`**를 돌려준다. UI는 이를 "꺼짐"으로 보여 주고 켜기를 허용한다. `/Applications`에 설치한 번들에서 `register`하면 바로 `Enabled`가 되고 시스템 설정 › 로그인 항목에 나타난다. 번들 실행 파일을 터미널에서 `--login-item status|on|off`로 불러 진단할 수 있다(`src-tauri/src/cli.rs`).

주의: `SMAppService`는 `.app` 번들에서 실행될 때 정상 동작한다. `tauri dev`는 번들 없이 실행 파일을 띄우므로 iyagi-term의 runner가 하는 `.app` 래핑(§4-6)을 그대로 쓴다. 래퍼로 등록한 로그인 항목은 dev 빌드 경로를 가리키므로, 릴리스 앱을 설치한 뒤에는 dev 쪽 등록을 해제해야 한다(트레이 토글로 해제, L07 체크리스트).

### 4-6. 코드 서명과 TCC 안정성 (가장 중요한 개발 경험 항목)

손쉬운 사용 권한은 앱의 **코드 서명 지정 요구 조건**에 묶인다. ad-hoc 서명은 빌드마다 cdhash가 바뀌어 macOS가 매번 새 앱으로 보고 권한을 다시 묻는다. iyagi-term이 이미 같은 문제(다른 앱 데이터 접근 TCC)를 풀어 두었다:

- `scripts/tauri.mjs`: 키체인에 로컬 인증서(기본 이름 `iyagi-dev`)가 있으면 dev/build를 그 인증서로 서명. build는 Developer ID 인증서가 하나면 그것을 우선하고 hardened runtime을 켠다.
- `scripts/macos-sign-runner.sh`: `cargo build` 직후 실행 파일을 복제본에 서명해 교체하고, dev에서는 `<제품명>.app` 래퍼로 감싼다. **Tauri CLI 2.12부터 dev가 runner를 `cargo run …`으로 부르므로**, 이 저장소의 복사본은 `run`을 `cargo build` + 서명·래핑 + `exec`로 바꿔 처리한다(iyagi-term이 CLI를 올리면 같은 수정이 필요하다).

이 둘을 복사해 다음만 바꾼다: 식별자 `ai.iyagi.layout`, 데몬 서명 줄 삭제, dev 래퍼 `Info.plist`에 `LSUIElement` 추가. 환경 변수 이름은 `IYAGI_*` 그대로 둔다(형제와 같은 셸 설정으로 동작).

이 Mac의 현재 상태(2026-10-09): 코드 서명 인증서로 로컬 인증서 1개(`moai-dev`)와 Developer ID Application 1개가 있다. `iyagi-dev` 인증서를 하나 만들거나 `IYAGI_SIGNING_IDENTITY=moai-dev`로 지정한다(두 형제가 같은 로컬 인증서를 써도 식별자가 달라 TCC 항목은 분리된다).

릴리스: Developer ID + hardened runtime → `xcrun notarytool submit --keychain-profile iyagi-notary --wait` → `xcrun stapler staple` → DMG. 절차는 `iyagi-term/releases/README.md`와 같다. **App Sandbox entitlement는 넣지 않는다.**

### 4-7. 플랫폼 요구

- 최소 macOS 13.0 (`bundle.macOS.minimumSystemVersion`. SMAppService 때문). 검증 환경은 macOS 26.6(이 Mac), Xcode 26.2, Rust 1.92, Node 24.
- Apple Silicon 우선. Intel은 요청 시 CI에서 cross build만.

## 5. 아키텍처와 저장소 구조

### 5-1. 디렉터리

```text
iyagi-layout/
├── Cargo.toml                  # workspace: crates/*, src-tauri (iyagi-term과 같은 형태)
├── package.json                # @tauri-apps/cli, react, vite (설정 창)
├── README.md / README_KR.md
├── IMPLEMENTATION_PLAN.md      # 이 문서
├── src-tauri/                  # crate iyagi-layout-app — Tauri 셸
│   ├── tauri.conf.json
│   ├── Info.plist              # LSUIElement 등, 번들 Info.plist에 병합
│   ├── capabilities/           # settings 창 권한
│   ├── icons/                  # 앱 아이콘 + 트레이 템플릿 아이콘
│   └── src/
│       ├── main.rs             # iyagi_layout_app_lib::run()
│       ├── lib.rs              # Builder, 플러그인, setup, RunEvent 처리
│       ├── tray.rs             # 트레이 메뉴 구성·이벤트
│       ├── hotkeys.rs          # 설정 → 단축키 등록/해제/재등록
│       ├── actions.rs          # 단축키 → core 계산 → macos 적용 (한 키 입력의 전체 흐름)
│       └── settings.rs         # settings.json 로드·검증·저장, 변경 통지
├── index.html / src/           # 설정 창 (React + Vite)
├── crates/
│   ├── layout-core/            # OS 의존 없음: 기하, 액션, 순환 상태, 설정 모델 (어느 OS에서나 테스트)
│   ├── layout-macos/           # AX·NSScreen·SMAppService·권한 FFI 래퍼 (cfg(target_os = "macos"))
│   └── layout-probe/           # 개발용 CLI: `cargo run -p layout-probe -- left` (Tauri 없이 AX 경로 검증)
├── scripts/
│   ├── tauri.mjs               # iyagi-term에서 복사·수정
│   ├── macos-sign-runner.sh    # iyagi-term에서 복사·수정 (LSUIElement 래퍼, 데몬 줄 삭제)
│   ├── generate_icons.py       # 자리표시 아이콘 + 트레이 템플릿 PNG
│   └── macos_probes.sh         # 수동 QA 보조 (대상 앱 띄우고 probe 실행)
├── docs/
│   └── implementation/         # 구현 중 확정한 계약 (예: 01-ax-window-frame.md)
├── releases/                   # DMG + README (iyagi-term 규칙)
└── .github/workflows/ci.yml
```

### 5-2. crate 책임

| crate | 의존 | 책임 | 테스트 |
|---|---|---|---|
| `layout-core` | serde만 | `Rect`/`Screen` 타입, `Action`(Left/Right/Up/Down …), `compute_target(action, screen_visible, current, gap, cycle)`, 좌표 변환, 화면 선택, 순환 상태 기계, `Settings` 모델·기본값·검증 | 순수 단위 테스트(모든 OS) |
| `layout-macos` | objc2-app-kit, objc2-application-services, objc2-service-management, objc2-core-foundation, Carbon(FFI) | `hotkey::{parse, register, unregister_all}`, `permission::{is_trusted, prompt, open_settings}`, `screens::list() -> Vec<Screen>`(CG 좌표로 변환 완료), `window::focused() -> Option<WindowHandle>`, `WindowHandle::{frame, set_frame, is_resizable, is_fullscreen, app_bundle_id}`, `login_item::{status, set_enabled}` | `#[ignore]` 통합 테스트(권한 필요) + probe |
| `layout-probe` | core + macos | `permission`, `screens`, `focused`, `left|right|up|down` 하위 명령으로 각 단계 출력 | 수동 |
| `iyagi-layout-app` | tauri 2.12, plugin-single-instance 2.5, plugin-opener, core + macos, tracing | 트레이, 단축키 등록, 설정 파일, 설정 창 명령, 생명주기 | 설정 파싱 등 단위 테스트 + 수동 QA |

### 5-3. 한 키 입력의 흐름

```text
Ctrl+Cmd+← 누름
  → Carbon 핫키 → global-hotkey → plugin 핸들러(메인 스레드, Pressed)
  → actions::run(Action::Left)
      1. layout_macos::permission::is_trusted()  없으면 → 안내(실행당 1회) + 트레이 상태 갱신, 끝
      2. layout_macos::window::focused()          없으면 → 로그, 끝
      3. frame = window.frame(); fullscreen이면 끝; resizable 여부 확인
      4. screens = layout_macos::screens::list()  (CG 좌표)
      5. screen = layout_core::pick_screen(frame, &screens)
      6. target = layout_core::compute_target(Left, screen.visible, frame, settings.gap, cycle_state)
      7. window.set_frame(target)   (enhancedUI 처리 → 크기 → 위치 → 크기 → 재읽기 → 보정)
      8. cycle_state.record(Left, target); tracing::info!(app = bundle_id, from = ?frame, to = ?target)
```

## 6. 동작 명세 (MVP)

### 6-1. 기본 단축키

기본값은 **⌥⌘(option+command)+방향키**다. 요구사항 문구는 "ctrl+cmd"였지만 실기 QA(2026-10-09, NSEvent 전역 모니터로 확인)에서 사용자가 실제로 누르는 조합은 맥북 키보드의 ⌘ 옆 키인 option이었다. 설정 창에서 언제든 바꿀 수 있다.

| 설정 키 | 기본값(저장 형식) | 동작 |
|---|---|---|
| `shortcuts.left` | `Alt+Super+ArrowLeft` | 왼쪽 절반 |
| `shortcuts.right` | `Alt+Super+ArrowRight` | 오른쪽 절반 |
| `shortcuts.up` | `Alt+Super+ArrowUp` | 위 절반 |
| `shortcuts.down` | `Alt+Super+ArrowDown` | 아래 절반 |

후속 후보(기본 미등록): `maximize` `Control+Super+Enter`, `restore` `Control+Super+Backspace`, `center` `Control+Super+KeyC`, 1/4 배치, 옆 모니터 이동 `Control+Super+Alt+ArrowLeft/Right`.

### 6-2. 목표 프레임 계산

`v` = 대상 화면의 `visibleFrame`(CG 좌표), `g` = 간격(기본 0, 허용 0–32):

| 액션 | x | y | w | h |
|---|---|---|---|---|
| Left | `v.x + g` | `v.y + g` | `(v.w - 3g) / 2` | `v.h - 2g` |
| Right | `v.x + g + w + g` | `v.y + g` | `(v.w - 3g) / 2` | `v.h - 2g` |
| Up | `v.x + g` | `v.y + g` | `v.w - 2g` | `(v.h - 3g) / 2` |
| Down | `v.x + g` | `v.y + g + h + g` | `v.w - 2g` | `(v.h - 3g) / 2` |

`g`는 화면 가장자리 여백과 창 사이 간격에 같은 값을 쓴다(분리는 후속). 모든 값은 계산 후 정수로 반올림하고, 두 절반의 합이 `v`를 넘지 않도록 w·h는 내림한다.

### 6-3. 연속 입력: 다시 나누기와 순환

연속 여부는 창 식별이 아니라 **직전에 우리가 둔 프레임에 창이 아직 그대로 있는가**(±1pt)로 판정한다. 사용자가 창을 손으로 옮기거나 다른 창으로 바꾸면 자연히 초기화된다. 상태는 `(action, last_frame, fraction, at)` 하나만 메모리에 둔다.

| 상황 | 계산 | 예 |
|---|---|---|
| 직전 배치 후 `chain_window_ms`(기본 3000, 0이면 끔) 안에 **축이 다른** 키 | 화면이 아니라 **현재 창**을 반으로 나눈다(`compute_chained`, 바깥 여백은 유지하고 조각 사이 간격만 추가) | ⌥⌘← 다음 ⌥⌘↓ = 왼쪽 아래 1/4, 이어서 ⌥⌘→ = 그 1/4의 오른쪽 절반 |
| 같은 축의 키(← 다음 →, 또는 같은 키) | 화면 기준 절반. 좌우를 빠르게 번갈아 눌러도 항상 화면 절반 | ⌥⌘← ⌥⌘→ ⌥⌘← … |
| 같은 키 반복 + `cycle_thirds`(기본 off) | `1/2 → 2/3 → 1/3 → 1/2 …` 순환. 시간 제한 없음 | ⌥⌘← ⌥⌘← ⌥⌘← |

이 규칙은 사용자가 2026-10-09 실기 중에 요청한 것이다("왼쪽 반으로 줄인 뒤 3초 안에 아래를 누르면 다시 반으로 줄여 아래로").

### 6-4. 대상 창과 화면 선택

- 대상 창: §4-2의 순서. 최소화된 창은 `AXFocusedWindow`가 아니므로 대상이 되지 않는다.
- 전체 화면 창: 무시. 크기 고정 창: 위치만 목표 영역의 왼쪽 위(+g)로 이동.
- 대상 화면: §4-3의 규칙. MVP에서는 창이 이미 그 절반을 차지하고 있어도 같은 화면 안에서 다시 적용한다(D5-a).

### 6-5. 권한 없음·실패 처리

| 상황 | 동작 |
|---|---|
| 손쉬운 사용 권한 없음 | 실행당 1회 시스템 안내창(`kAXTrustedCheckOptionPrompt`), 트레이 상태 줄 "권한 필요", 메뉴 "손쉬운 사용 설정 열기" |
| 단축키 등록 실패 | 트레이 상태 줄에 실패한 키 표시, 로그 warn, 나머지는 정상 등록 |
| 포커스 창을 못 찾음(앱 전환 직후 포함) | 150ms 간격으로 최대 10회 재시도(약 1.5초). 끝까지 없으면 알림 "포커스된 창이 없습니다". 로그에 최전면 앱 번들 ID를 남긴다 |
| 전체 화면 창, 또는 IYAGI Layout 자신의 창이 앞에 있음 | 옮기지 않는다. 알림 "전체 화면 창은 옮길 수 없습니다" / "다른 앱의 창을 먼저 선택하세요" |
| 창 위치·크기 읽기·쓰기 실패 | 로그 warn. 알림 "창을 옮기지 못했습니다". 절대 패닉하지 않는다 |
| 설정 파일 손상 | 기본값으로 동작 + 로그 warn + 트레이 상태 줄 표시. 파일을 덮어쓰지 않는다 |

알림은 메뉴 막대 아이콘 옆 제목에 `⚠︎ 문구`로 3초 띄우고, 상태 줄에는 마지막 실패를 다음 성공 때까지 남긴다. 키를 누른 순간에 보이는 유일한 신호이므로 로그만 남기지 않는다.

### 6-6. 설정 파일

경로: `~/Library/Application Support/ai.iyagi.layout/settings.json` (Tauri `app_config_dir`). 없으면 기본값으로 생성.

```json
{
  "version": 1,
  "shortcuts": {
    "left": "Alt+Super+ArrowLeft",
    "right": "Alt+Super+ArrowRight",
    "up": "Alt+Super+ArrowUp",
    "down": "Alt+Super+ArrowDown"
  },
  "gap": 0,
  "cycle_thirds": false,
  "chain_window_ms": 3000,
  "enhanced_ui_policy": "automatic",
  "log_level": "info"
}
```

로그인 항목 여부는 설정 파일에 두지 않는다(OS가 진실의 원천, §4-5). 파일을 외부에서 고치면 다음 실행에 반영된다(파일 감시는 후속. 트레이 "설정 다시 읽기" 항목으로 대신한다).

### 6-7. 로그

`~/Library/Logs/ai.iyagi.layout/iyagi-layout.log.<날짜>`(Tauri `app_log_dir`), `tracing` + `tracing-appender`(일 단위 회전, iyagi-termd와 같은 조합). 키 입력마다 1줄(앱 번들 ID, 이전/이후 프레임, 소요 ms). 창 제목은 기록하지 않는다(개인정보).

### 6-8. 트레이 메뉴 (MVP)

설정 창이 MVP에 들어왔으므로(D3-b) 트레이는 진입점과 상태만 둔다.

```text
상태 줄 (비활성): 권한 없음 → 설정 파일 오류 → 단축키 등록 실패 → 정상이면 "⌃⌘← 왼쪽 절반 · …"
──────────
설정… · Settings…                       (⌘,)
손쉬운 사용 설정 열기 · Accessibility Settings
☑ 로그인 시 실행 · Launch at Login       (SMAppService status로 갱신)
로그 폴더 열기 · Open Logs
──────────
IYAGI Layout 0.1.0                       (비활성)
종료 · Quit                              (⌘Q)
```

아이콘은 메뉴 막대 템플릿 이미지(흑백+알파, `icon_as_template(true)`). iyagi-term처럼 한/영을 병기한다.

## 6-9. 다국어 (i18n)

- 지원: 영어(en), 한국어(ko), 일본어(ja), 중국어 간체(zh). 번체 지역(zh-TW, zh-HK)도 간체로 표시한다.
- 언어 선택: 시스템 선호 언어 목록(시스템 설정 › 일반 › 언어 및 지역)에서 **지원하는 첫 언어**, 하나도 없으면 영어(macOS 앱의 표준 동작). 예: `fr, ja, ko` → 일본어, `de, es` → 영어.
- Rust 쪽(메뉴 막대 메뉴·상태 줄): `layout-core::i18n`(`Lang`, `Msg`, `tr`), 시스템 언어는 `layout-macos::locale`(NSLocale.preferredLanguages). 프로세스 시작 시 한 번 정하므로 시스템 언어를 바꾸면 앱을 다시 실행해야 메뉴 막대에 반영된다.
- 설정 창: `src/i18n.ts` 사전 4개. 언어는 앱이 `app_info.lang`으로 알려 주는 값(메뉴 막대와 동일)을 쓰고, 응답 전에는 `navigator.languages`로 그린다. 사전은 영어 키를 기준으로 타입 검사해 번역 누락이 컴파일 오류가 된다.
- 시험: `IYAGI_LANG=ko|en|ja|zh`로 언어를 강제할 수 있고, `iyagi-layout-app --lang`이 감지 결과와 샘플 문구를 출력한다. 단위 테스트는 태그 매핑·우선순위·번역 누락을 검사한다.
- 로그 메시지와 설정 파일 키는 번역하지 않는다(영어 고정).

## 7. 설정 창 (MVP, D3-b)

React 18 + Vite + TypeScript(iyagi-term과 같은 버전), 창 label `settings`, 트레이 "설정…"에서 지연 생성, 닫으면 숨김(`CloseRequested` → `prevent_close` + `hide`). 명령: `settings_get`, `settings_set`, `permission_status`, `permission_prompt`, `open_accessibility_settings`, `login_item_status`, `login_item_set`; 이벤트 `settings-changed`. 단축키 녹화기(키 조합 입력 → 파서 검증 → 즉시 재등록)가 핵심. 구성: 권한 상태 카드, 단축키 4개 녹화기, 간격, 순환 옵션, 로그인 시 실행 토글, 로그 폴더 열기.

## 8. 빌드·서명·배포

### 8-1. `src-tauri/tauri.conf.json` 핵심

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "IYAGI Layout",
  "version": "0.1.0",
  "identifier": "ai.iyagi.layout",
  "build": {},
  "app": { "windows": [] },
  "bundle": {
    "active": true,
    "license": "Apache-2.0",
    "targets": ["app", "dmg"],
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/128x128@2x.png", "icons/icon.icns"],
    "macOS": { "minimumSystemVersion": "13.0", "hardenedRuntime": true }
  }
}
```

`src-tauri/Info.plist`:

```xml
<plist version="1.0"><dict>
  <key>LSUIElement</key><true/>
  <key>NSHumanReadableCopyright</key><string>Apache-2.0</string>
</dict></plist>
```

### 8-2. 의존성 버전 (2026-10-09 기준 최신 안정)

| 패키지 | 버전 |
|---|---|
| tauri / tauri-build | 2.12.2 / 2.7.1 |
| tauri-plugin-single-instance | 2.5.2 |
| objc2-app-kit / objc2-application-services / objc2-service-management / objc2-core-foundation | 0.3.2 (Tauri 2.12가 쓰는 objc2 0.6 계열과 동일) |
| @tauri-apps/cli | 2.12.1 |
| (선택) tauri-plugin-autostart | 2.7.0 (D2-b/c 선택 시) |

### 8-3. 개발·빌드 명령

```sh
npm ci
npm run tauri dev                       # scripts/tauri.mjs → 로컬 인증서 서명 + "IYAGI Layout.app" 래퍼
cargo run -p layout-probe -- focused    # Tauri 없이 AX 경로 확인
npm run tauri build                     # Developer ID가 있으면 자동 선택, 없으면 ad-hoc
```

### 8-4. 릴리스

`releases/IYAGI.Layout_<version>_aarch64.dmg` + README에 SHA-256(iyagi-term 규칙). 공증·스테이플 절차는 §4-6.

### 8-5. CI (`.github/workflows/ci.yml`)

| job | runner | 내용 |
|---|---|---|
| `core-check` | ubuntu-latest(컨테이너) | `cargo fmt/clippy/test -p layout-core` (OS 의존 없음, 저렴) |
| `macos-check` | **GitHub-hosted macos-latest** | fmt, clippy `--workspace`, test, `npm run tauri build -- --no-bundle` (비밀값 없음, PR에서도 실행) |
| `bundle-macos` | self-hosted macos, 보호 ref만 | 서명 DMG 생성·업로드 (iyagi-term과 같은 보안 정책 주석 유지) |

iyagi-term과 달리 macOS 전용이라 Windows/Linux 번들 job이 없다. macOS check를 GitHub-hosted로 돌릴 수 있는 이유는 서명·권한이 필요 없기 때문이다(AX 통합 테스트는 `--ignored`라 CI에서 돌지 않는다).

## 9. 검증과 완료 기준

### 9-1. 자동 테스트

- `layout-core`: 절반 계산(gap 0/8/홀수 폭), 정수 반올림과 합 ≤ 화면, Cocoa↔CG 변환(주 화면/보조 화면/위쪽 배치 모니터), 화면 선택(교집합/중심/없음), 순환 상태 기계(연속·중단·옵션 off), 설정 파싱(기본값, 모르는 키 무시, 잘못된 단축키 문자열 거부).
- `layout-macos`: `#[ignore]` 통합 테스트 — `open -a TextEdit`로 창을 띄운 뒤 포커스 창 프레임 읽기/쓰기/되돌리기. 로컬에서 `cargo test -p layout-macos -- --ignored`.
- `iyagi-layout-app`: 설정 로드/저장, 단축키 문자열 → 플러그인 파서 검증(실제 등록 없이 `Shortcut::from_str`).

### 9-2. 수동 QA 매트릭스 (L08 완료 조건)

| 축 | 항목 |
|---|---|
| 대상 앱 | Terminal, iTerm2, IYAGI Term(WKWebView), VS Code·Chrome(Electron/Chromium), Safari, Xcode(단축키 충돌 확인), Finder, 시스템 설정(크기 고정), 미리보기 |
| 화면 | 노치 MacBook 단독 / 외장 모니터 2대(배율 다름, 위쪽에 배치) / 연결·해제 직후 |
| 창 상태 | 전체 화면(무시), 최소화(무시), Stage Manager 켬, 다른 Space |
| 생명주기 | 트레이 종료 → 단축키 해제됨, 두 번 실행 → 하나만, 로그인 항목 on → 재부팅 → 트레이만 뜸(창·Dock 없음), 설정 앱에서 로그인 항목 끔 → 메뉴에 반영 |
| 권한 | 미허용 → 안내 1회, 허용 후 즉시 동작, 허용 취소 → 다시 안내, **재빌드 후 권한 유지** |

### 9-3. 완료 기준 (MVP)

1. §1-2의 8개 항목이 §9-2 매트릭스에서 통과한다.
2. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, 모든 단위 테스트 통과.
3. 서명·공증된 DMG가 새 Mac(또는 새 사용자 계정)에서 Gatekeeper 경고 없이 열리고, 권한 안내 → 허용 → 동작까지 이어진다.
4. README(한/영)에 설치, 권한, 단축키, 로그인 항목, 제거 방법(로그인 항목 해제 포함)이 있다.

## 10. 구현 티켓

| 티켓 | 크기 | 상태(2026-10-09) | 내용 | 완료 조건 |
|---|---|---|---|---|
| L01 골격 | M | ✅ | 워크스페이스, Tauri 셸, `Info.plist` LSUIElement, 스크립트 복사·수정(CLI 2.12 `run` 대응), 아이콘 자리표시 | `tauri dev`가 `IYAGI Layout.app` 래퍼로 Dock 없이 뜬다. 안정 서명은 `iyagi-dev` 인증서가 생기면 자동 적용(현재 ad-hoc) |
| L02 core | M | ✅ | `layout-core` 타입·계산·순환·설정 모델 + 테스트 14개 | 통과 |
| L03 macos 읽기 | M | ✅ | 권한, 화면 목록(CG 좌표), 포커스 창 읽기, probe | 2 모니터(노치 1710×1107 + 2560×1440)에서 화면 목록 확인. 창 읽기는 권한 부여 후 확인 필요 |
| L04 macos 쓰기 | M | ✅ 코드 | `set_frame`(enhancedUI, 크기→위치→크기, 재읽기 보정), fullscreen/resizable 판정 | **권한 부여 후 실기 QA 필요** |
| L05 단축키 | S | ✅ | 플러그인 등록, 메인 스레드 디스패치, 실패 기록, 설정 변경 시 재등록 | 4키 등록 로그 확인 |
| L06 설정 | S | ✅ | settings.json 로드/검증/원자적 저장, 손상 시 기본값 | 기본 파일 생성 확인 |
| L07 로그인 항목 | S | ✅ 코드 | SMAppService status/register/unregister, 트레이·설정 창 토글 | **재부팅 QA 필요**(dev 래퍼로 등록 시 dev 경로를 가리킨다) |
| L10 설정 창 | L | ✅ | 권한 카드(2초마다 갱신), 단축키 녹화기(실제 파서 검증), 간격·순환, 로그인 항목, 로그 폴더 | UI 실기 확인 필요 |
| L08 생명주기·QA | M | 🔶 | Chrome에서 4방향·연속 분할 실기 확인. 수동 실행 시 설정 창 표시, 로그인 자동 실행 시 메뉴 막대만(`launch::launched_at_login` 어림 규칙: 세션 로그인 120초 이내). 나머지 §9-2 매트릭스 수행 남음 | 매트릭스 통과 |
| L09 릴리스 | M | ✅ | 실제 아이콘(`icons/source/layout.png` → `scripts/generate_icons.py`가 RGBA PNG·ICNS 생성; tauri-codegen은 RGBA만 받는다). **macOS 26 아이콘**: .icns만 있으면 Tahoe가 회색 둥근 사각형 안에 흐릿하게 그리므로 `icons/AppIcon.icon`(Icon Composer 형식)을 `scripts/build-icon-assets.sh`가 actool로 `Assets.car`로 컴파일해 `bundle.macOS.files`로 넣고 Info.plist에 `CFBundleIconName=AppIcon`을 둔다(Tauri 2.12 미지원 기능의 우회). Developer ID 서명·공증·스테이플 완료, `releases/IYAGI.Layout_0.1.0_aarch64.dmg` 등록(2026-10-10), `/Applications`에서 로그인 항목 등록 확인 | §9-3 충족 |
| L11 확장 액션 | M | 🔲 후속 | 최대화/복원/중앙/1/4/옆 모니터 | — |

순서는 L01 → L02 → L03 → L04 → L05 → L06 → L07 → L10 → L08 → L09(L10이 MVP에 들어왔다). L02는 L01과 병행 가능. L03·L04는 probe 덕분에 Tauri 셸 없이 검증한다.

## 11. 리스크와 대응

| 리스크 | 영향 | 대응 |
|---|---|---|
| Carbon `RegisterEventHotKey`는 오래된 API | Apple이 제거하면 단축키 불가 | 현재 macOS 26에서 동작(Tauri/global-hotkey가 의존). `hotkeys.rs` 뒤에 백엔드를 감춰 `CGEventTap`(입력 모니터링 권한 필요)으로 교체 가능하게 둔다 |
| 앱별 AX 특이 동작(Electron, JetBrains, 고정 크기) | 일부 앱에서 어긋남 | 재읽기 보정 + 앱별 우회를 `layout-macos::quirks`에 격리, QA 매트릭스에 앱 추가 |
| TCC 권한이 서명 변화로 초기화 | 개발 생산성 급락 | §4-6의 안정 서명을 L01의 완료 조건으로 못 박는다 |
| 로그인 항목 승인 알림·사용자 비활성화 | "안 뜬다" 문의 | `status()`를 메뉴에 그대로 표시, README에 설정 경로 안내 |
| 응답 없는 대상 앱이 메인 스레드를 막음 | 트레이가 잠시 멈춤 | 메시징 타임아웃 0.5초. 체감되면 워커 스레드 |
| Tauri 3 전환 | 플러그인 API 변경 | 2.x 고정, iyagi-term과 함께 이전 |
| 단축키 충돌(Xcode 등) | 해당 앱 단축키 가려짐 | 설정 파일로 변경, 후속 설정 창 |

## 12. 참고

- Apple, "Mac에서 창 타일링" 단축키 표: https://support.apple.com/guide/mac-help/mchl9674d0b0/mac
- Rectangle (Swift, MIT) — AX 프레임 설정 순서·Enhanced UI 처리의 참조 구현: https://github.com/rxhanson/Rectangle
- tauri-plugin-global-shortcut: https://docs.rs/tauri-plugin-global-shortcut · global-hotkey 파서/백엔드 소스: https://docs.rs/crate/global-hotkey/0.8.0/source/
- objc2-application-services(AXUIElement/AXValue): https://docs.rs/objc2-application-services/0.3.2
- objc2-service-management(SMAppService): https://docs.rs/objc2-service-management/0.3.2
- tauri-plugin-autostart(대안): https://docs.rs/tauri-plugin-autostart
- Tauri 2 시스템 트레이(마지막 창이 닫혀도 유지하는 패턴): https://v2.tauri.app/learn/system-tray/
- iyagi-term 참조 파일: `scripts/tauri.mjs`, `scripts/macos-sign-runner.sh`, `src-tauri/src/lib.rs`(트레이·종료 흐름), `releases/README.md`(공증 절차), `.github/workflows/ci.yml`(보안 정책)
