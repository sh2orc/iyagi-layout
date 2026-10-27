//! `~/Library/Logs/ai.iyagi.layout/iyagi-layout.log.<date>` (일 단위 회전) + 개발
//! 빌드에서는 stderr. 로그 수준은 settings.json의 `log_level`. 기동 시
//! `LOG_RETENTION_DAYS`보다 오래된 로그는 지운다(회전은 파일만 만든다).

use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Manager};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

static GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();

const LOG_RETENTION_DAYS: u64 = 14;

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
        prune_old_logs(&dir);
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

/// 우리 로그 파일(`iyagi-layout.log.<date>`) 중 수정 시각이 보존 기간을 넘은 것 삭제.
/// 로거 초기화 전에 불리므로 tracing이 아니라 무시한다 — 실패해도 기동에는 지장 없다.
fn prune_old_logs(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(LOG_RETENTION_DAYS * 24 * 60 * 60))
        .unwrap_or(SystemTime::UNIX_EPOCH);
    let mut removed = 0u32;
    for entry in entries.flatten() {
        let is_ours = entry
            .file_name()
            .to_str()
            .is_some_and(|n| n.starts_with("iyagi-layout.log."));
        if !is_ours {
            continue;
        }
        let stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .is_ok_and(|modified| modified < cutoff);
        if stale && std::fs::remove_file(entry.path()).is_ok() {
            removed += 1;
        }
    }
    if removed > 0 {
        eprintln!("pruned {removed} log file(s) older than {LOG_RETENTION_DAYS} days");
    }
}
