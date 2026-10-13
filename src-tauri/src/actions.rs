//! 단축키 한 번 → 창 배치 한 번(§5-3). 메인 스레드에서 실행한다(NSScreen).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use layout_core::i18n::Msg;
use layout_core::{
    compute_chained, compute_target, pick_screen, Action, CycleState, Fraction, Plan, Rules,
};
use tauri::{AppHandle, Manager};

/// 포커스 창을 못 찾으면 이 간격으로 다시 본다. 앱을 바꾸는 순간 macOS가 잠깐 "포커스 없음"을 돌려준다.
const RETRY_INTERVAL: Duration = Duration::from_millis(150);
/// 재시도 횟수. 150ms × 10 = 최대 1.5초.
const MAX_RETRIES: u32 = 10;

#[derive(Default)]
pub struct ActionState {
    cycle: Mutex<CycleState>,
    /// 실행당 한 번만 시스템 권한 안내창을 띄운다.
    prompted: AtomicBool,
    /// 마지막 실행이 실패했다면 이유. 메뉴 막대 상태 줄에 보이고, 다음 성공 때 비운다.
    last_failure: Mutex<Option<Msg>>,
}

impl ActionState {
    pub fn last_failure(&self) -> Option<Msg> {
        *self.last_failure.lock().expect("failure lock")
    }
}

/// 메뉴 막대 상태 줄과 제목에 쓰는 실패 문구.
pub fn failure_label(msg: Msg) -> String {
    format!("⚠︎ {}", crate::i18n::t(msg))
}

pub fn run(app: &AppHandle, action: Action) {
    attempt(app, action, Instant::now(), 0);
}

/// 한 번의 시도. `started`는 키를 누른 시각이라 재시도해도 그대로 둔다(순환·이어서 분할 판단에 쓴다).
fn attempt(app: &AppHandle, action: Action, started: Instant, tries: u32) {
    let settings = app.state::<crate::settings::SettingsState>().current();
    let state = app.state::<ActionState>();

    if !layout_macos::permission::is_trusted() {
        if !state.prompted.swap(true, Ordering::SeqCst) {
            tracing::warn!("accessibility permission missing — prompting once");
            layout_macos::permission::prompt();
        } else {
            tracing::warn!("accessibility permission missing — key ignored");
        }
        crate::tray::refresh_status(app);
        return;
    }

    let window = match layout_macos::window::focused() {
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
                    front = ?layout_macos::window::frontmost_bundle_id(),
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
    let screens = match layout_macos::screens::list() {
        Ok(s) => s,
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
        .expect("cycle lock")
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
                .expect("cycle lock")
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

/// 메인 스레드를 막지 않으려고 타이머 스레드에서 기다렸다가, 메인 스레드에서 다시 시도한다.
fn retry(app: &AppHandle, action: Action, started: Instant, tries: u32) {
    tracing::debug!(?action, tries, "no focused window yet — retrying");
    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(RETRY_INTERVAL);
        let next = handle.clone();
        let _ = handle.run_on_main_thread(move || attempt(&next, action, started, tries + 1));
    });
}

/// 실패 이유를 남기고 메뉴 막대에 잠깐 알린다. 키를 누른 순간에 보이는 유일한 신호다.
fn fail(app: &AppHandle, msg: Msg) {
    *app.state::<ActionState>()
        .last_failure
        .lock()
        .expect("failure lock") = Some(msg);
    crate::tray::refresh_status(app);
    crate::tray::flash(app, &failure_label(msg));
}

/// 성공하면 이전 실패 문구를 지운다.
fn clear_failure(app: &AppHandle) {
    let cleared = app
        .state::<ActionState>()
        .last_failure
        .lock()
        .expect("failure lock")
        .take()
        .is_some();
    if cleared {
        crate::tray::refresh_status(app);
    }
}
