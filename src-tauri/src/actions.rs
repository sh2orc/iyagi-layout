//! 단축키 한 번 → 창 배치 한 번(§5-3). Carbon 핸들러는 **메인 스레드 전용 값**
//! (NSScreen 화면 목록, NSWorkspace 최전면 앱)만 모아 워커 스레드에 넘긴다. AX 호출은
//! 응답 없는 앱을 만나면 하나당 0.5초까지 기다릴 수 있어, 워커에서 실행해야 메인 루프
//! (트레이·설정 창)가 멈추지 않는다.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use layout_core::i18n::Msg;
use layout_core::{
    compute_chained, compute_target, pick_screen, Action, CycleState, Fraction, Plan, Rules, Screen,
};
use tauri::{AppHandle, Manager};

/// 포커스 창을 못 찾으면 이 간격으로 다시 본다. 앱을 바꾸는 순간 macOS가 잠깐 "포커스 없음"을 돌려준다.
const RETRY_INTERVAL: Duration = Duration::from_millis(150);
/// 재시도 횟수. 150ms × 10 = 최대 1.5초.
const MAX_RETRIES: u32 = 10;

#[derive(Default)]
pub struct ActionState {
    cycle: Mutex<CycleState>,
    /// 스냅은 한 번에 하나만 — 워커 스레드가 늘어나도 AX 읽기·쓰기가 겹치지 않게 한다.
    /// 예전(메인 스레드 동기 실행)과 같은 직렬화 규칙이다.
    serial: Mutex<()>,
    /// 실행당 한 번만 시스템 권한 안내창을 띄운다.
    prompted: AtomicBool,
    /// 마지막 실행이 실패했다면 이유. 메뉴 막대 상태 줄에 보이고, 다음 성공 때 비운다.
    last_failure: Mutex<Option<Msg>>,
}

impl ActionState {
    pub fn last_failure(&self) -> Option<Msg> {
        *self
            .last_failure
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// 메뉴 막대 상태 줄과 제목에 쓰는 실패 문구.
pub fn failure_label(msg: Msg) -> String {
    format!("⚠︎ {}", crate::i18n::t(msg))
}

pub fn run(app: &AppHandle, action: Action) {
    start(app, action, Instant::now(), 0);
}

/// 메인 스레드에서만 얻을 수 있는 값. 재시도할 때마다 새로 모은다.
struct MainSnapshot {
    screens: Result<Vec<Screen>, layout_macos::AxError>,
    /// NSWorkspace 최전면 앱 PID(크롬처럼 AX 포커스 앱을 비우는 앱의 폴백용).
    frontmost_pid: Option<i32>,
    /// 최전면 앱 번들 ID — "포커스 창 없음" 로그에 남긴다.
    frontmost_bundle: Option<String>,
}

impl MainSnapshot {
    fn take() -> Self {
        let frontmost = layout_macos::window::frontmost_app_info();
        Self {
            screens: layout_macos::screens::list(),
            frontmost_pid: frontmost.as_ref().map(|(pid, _)| *pid),
            frontmost_bundle: frontmost.and_then(|(_, bundle)| bundle),
        }
    }
}

/// 메인 스레드에서: 스냅샷을 모아 워커를 띄운다. Carbon 핸들러가 여기서 바로 돌아간다.
fn start(app: &AppHandle, action: Action, started: Instant, tries: u32) {
    let snapshot = MainSnapshot::take();
    let handle = app.clone();
    let spawned = std::thread::Builder::new()
        .name("snap".into())
        .spawn(move || attempt(&handle, action, started, tries, snapshot));
    if let Err(error) = spawned {
        tracing::warn!(%error, "cannot spawn the snap worker");
        fail(app, Msg::MoveFailed);
    }
}

/// 한 번의 시도(워커 스레드). `started`는 키를 누른 시각이라 재시도해도 그대로 둔다(순환·이어서 분할 판단에 쓴다).
fn attempt(app: &AppHandle, action: Action, started: Instant, tries: u32, snapshot: MainSnapshot) {
    let settings = app.state::<crate::settings::SettingsState>().current();
    let state = app.state::<ActionState>();
    // 진행 중인 스냅이 있으면 끝날 때까지 기다린다(기다림의 상한은 AX 타임아웃의 합).
    let _one_at_a_time = state
        .serial
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if !layout_macos::permission::is_trusted() {
        let handle = app.clone();
        if !state.prompted.swap(true, Ordering::SeqCst) {
            tracing::warn!("accessibility permission missing — prompting once");
            // 안내창 띄우기도 메인 스레드로 넘긴다.
            let _ = app.run_on_main_thread(move || {
                layout_macos::permission::prompt();
                crate::tray::refresh_status(&handle);
            });
        } else {
            tracing::warn!("accessibility permission missing — key ignored");
            let _ = app.run_on_main_thread(move || crate::tray::refresh_status(&handle));
        }
        return;
    }

    let window = match layout_macos::window::focused_with_frontmost(snapshot.frontmost_pid) {
        Ok(Some(w)) => w,
        outcome => {
            if tries < MAX_RETRIES {
                retry(app, action, started, tries);
                return;
            }
            match outcome {
                Err(error) => {
                    tracing::warn!(?action, tries, %error, "cannot find the focused window")
                }
                _ => tracing::info!(
                    ?action,
                    tries,
                    front = ?snapshot.frontmost_bundle,
                    "no focused window"
                ),
            }
            fail(app, Msg::NoFocusedWindow);
            return;
        }
    };
    // 설정 창이 앞에 있으면 옮기지 않는다. 다른 앱의 창을 고르라고 안내한다.
    if window.pid == std::process::id() as i32 {
        tracing::info!(?action, "own window is focused — nothing to move");
        fail(app, Msg::OwnWindow);
        return;
    }
    if window.is_fullscreen() {
        tracing::info!(?action, app = ?window.bundle_id, "full-screen window — skipped");
        fail(app, Msg::FullScreenSkipped);
        return;
    }
    let frame = match window.frame() {
        Ok(f) => f,
        Err(error) => {
            tracing::warn!(?action, %error, "cannot read the window frame");
            fail(app, Msg::MoveFailed);
            return;
        }
    };
    let screens = match snapshot.screens {
        Ok(s) if !s.is_empty() => s,
        Ok(_) => {
            tracing::warn!("no screens");
            fail(app, Msg::MoveFailed);
            return;
        }
        Err(error) => {
            tracing::warn!(%error, "cannot list screens");
            fail(app, Msg::MoveFailed);
            return;
        }
    };
    let Some(screen) = pick_screen(&frame, &screens, None) else {
        tracing::warn!("no screens");
        fail(app, Msg::MoveFailed);
        return;
    };

    let rules = Rules {
        cycle_thirds: settings.cycle_thirds,
        chain_window: Duration::from_millis(u64::from(settings.chain_window_ms)),
    };
    let plan = state
        .cycle
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .plan(action, &frame, started, rules);
    let (target, fraction) = match plan {
        Plan::FromScreen(fraction) => (
            compute_target(action, screen.visible, settings.gap as f64, fraction),
            fraction,
        ),
        Plan::Chained => (
            compute_chained(action, frame, settings.gap as f64, Fraction::Half),
            Fraction::Half,
        ),
    };
    match window.set_frame(target, settings.enhanced_ui_policy) {
        Ok(actual) => {
            state
                .cycle
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .record(action, actual, fraction, started);
            clear_failure(app);
            tracing::info!(
                ?action,
                ?plan,
                app = ?window.bundle_id,
                screen = screen.id,
                from = ?frame,
                to = ?actual,
                ms = started.elapsed().as_millis() as u64,
                "placed"
            );
        }
        Err(error) => {
            tracing::warn!(?action, app = ?window.bundle_id, %error, "cannot set the window frame");
            fail(app, Msg::MoveFailed);
        }
    }
}

/// 재시도: 기다렸다가 메인 스레드로 돌아가(화면·최전면 앱을 새로 모으려고) 다시 시작한다.
fn retry(app: &AppHandle, action: Action, started: Instant, tries: u32) {
    tracing::debug!(?action, tries, "no focused window yet — retrying");
    let handle = app.clone();
    let spawned = std::thread::Builder::new()
        .name("snap-retry".into())
        .spawn(move || {
            std::thread::sleep(RETRY_INTERVAL);
            let next = handle.clone();
            let _ = handle.run_on_main_thread(move || start(&next, action, started, tries + 1));
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "cannot spawn the snap retry thread");
    }
}

/// 실패 이유를 남기고 메뉴 막대에 잠깐 알린다. 키를 누른 순간에 보이는 유일한 신호다.
/// 워커에서 불리므로 트레이 갱신은 메인 스레드로 넘긴다.
fn fail(app: &AppHandle, msg: Msg) {
    *app.state::<ActionState>()
        .last_failure
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(msg);
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        crate::tray::refresh_status(&handle);
        crate::tray::flash(&handle, &failure_label(msg));
    });
}

/// 성공하면 이전 실패 문구를 지운다.
fn clear_failure(app: &AppHandle) {
    let cleared = app
        .state::<ActionState>()
        .last_failure
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take()
        .is_some();
    if cleared {
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || crate::tray::refresh_status(&handle));
    }
}
