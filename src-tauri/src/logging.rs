//! `~/Library/Logs/ai.iyagi.layout/iyagi-layout.log.<date>` (일 단위 회전) + 개발
//! 빌드에서는 stderr. 로그 수준은 settings.json의 `log_level`.

use std::sync::OnceLock;

use tauri::{AppHandle, Manager};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

pub fn log_dir(app: &AppHandle) -> Option<std::path::PathBuf> {
    app.path().app_log_dir().ok()
}

pub fn init(app: &AppHandle) {
    let level = app
        .state::<crate::settings::SettingsState>()
        .current()
        .log_level;
    let filter = EnvFilter::try_new(format!(
        "iyagi_layout_app_lib={level},layout_macos={level},layout_core={level},warn"
    ))
    .unwrap_or_else(|_| EnvFilter::new("info"));

    let file_layer = log_dir(app).and_then(|dir| {
        std::fs::create_dir_all(&dir).ok()?;
        let appender = tracing_appender::rolling::daily(dir, "iyagi-layout.log");
        let (writer, guard) = tracing_appender::non_blocking(appender);
        let _ = GUARD.set(guard);
        Some(
            fmt::layer()
                .with_ansi(false)
                .with_target(false)
                .with_writer(writer),
        )
    });

    let registry = tracing_subscriber::registry().with(filter).with(file_layer);
    if cfg!(debug_assertions) {
        let _ = registry.with(fmt::layer().with_target(false)).try_init();
    } else {
        let _ = registry.try_init();
    }
}
