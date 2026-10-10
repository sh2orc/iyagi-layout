//! 단축키 한 번 → 창 배치 한 번(§5-3). 메인 스레드에서 실행한다(NSScreen).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use layout_core::{
    compute_chained, compute_target, pick_screen, Action, CycleState, Fraction, Plan, Rules,
};
use tauri::{AppHandle, Manager};

#[derive(Default)]
pub struct ActionState {
    cycle: Mutex<CycleState>,
    /// 실행당 한 번만 시스템 권한 안내창을 띄운다.
    prompted: AtomicBool,
}

pub fn run(app: &AppHandle, action: Action) {
    let started = Instant::now();
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
        Ok(None) => {
            tracing::info!(?action, "no focused window");
            return;
        }
        Err(error) => {
            tracing::warn!(?action, %error, "cannot find the focused window");
            return;
        }
    };
    if window.is_fullscreen() {
        tracing::info!(?action, app = ?window.bundle_id, "full-screen window — skipped");
        return;
    }
    let frame = match window.frame() {
        Ok(f) => f,
        Err(error) => {
            tracing::warn!(?action, %error, "cannot read the window frame");
            return;
        }
    };
    let screens = match layout_macos::screens::list() {
        Ok(s) => s,
        Err(error) => {
            tracing::warn!(%error, "cannot list screens");
            return;
        }
    };
    let Some(screen) = pick_screen(&frame, &screens, None) else {
        tracing::warn!("no screens");
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
        }
    }
}
