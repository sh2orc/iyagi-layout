//! Tauri 셸. 메뉴 막대 아이콘, 전역 단축키, 설정 창, 생명주기를 소유한다.
//! 창 계산은 `layout-core`, macOS 호출은 `layout-macos`가 맡는다.

use tauri::Manager;

mod actions;
mod cli;
mod hotkeys;
mod i18n;
mod logging;
mod settings;
mod system;
mod tray;
mod window;

pub fn run() {
    if cli::handle() {
        return;
    }
    tauri::Builder::default()
        // 로그인 항목 + 수동 실행이 겹쳐도 인스턴스는 하나: 두 번째 실행은 첫
        // 인스턴스의 설정 창을 띄우고 바로 끝난다. (문서 권고대로 가장 먼저 등록.)
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tracing::info!("second instance launched — showing the settings window instead");
            window::show_settings(app);
        }))
        .plugin(tauri_plugin_opener::init())
        .manage(settings::SettingsState::default())
        .manage(actions::ActionState::default())
        .manage(hotkeys::HotkeyState::default())
        .invoke_handler(tauri::generate_handler![
            settings::settings_get,
            settings::settings_set,
            settings::settings_path,
            window::app_info,
            window::open_log_dir,
            system::status_get,
            system::permission_prompt,
            system::open_accessibility_settings,
            system::login_item_set,
            system::open_login_items_settings,
            system::shortcut_validate,
            system::app_quit,
        ])
        .setup(|app| {
            // 메뉴 막대 전용(Info.plist의 LSUIElement를 번들 밖 dev 실행에서도 보강).
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            settings::load_into_state(app.handle());
            logging::init(app.handle());
            layout_macos::ax_timeout();
            hotkeys::apply(
                app.handle(),
                &app.state::<settings::SettingsState>().current(),
            );
            tray::setup(app)?;
            // 권한이 없으면 시스템 안내창을 띄운다 — 이 요청이 있어야 손쉬운 사용 목록에
            // 앱 항목이 생긴다.
            let trusted = layout_macos::permission::is_trusted();
            if !trusted {
                tracing::warn!("accessibility permission missing — prompting at startup");
                layout_macos::permission::prompt();
            }
            // 사용자가 직접 실행했으면 설정 창을 보여 준다. 로그인 직후 자동 실행이면
            // 조용히 메뉴 막대에만 머문다(권한이 없을 때는 예외 — 안내가 필요하다).
            let at_login = layout_macos::launch::launched_at_login();
            if !trusted || !at_login {
                window::show_settings(app.handle());
            } else {
                tracing::info!("launched at login — staying in the menu bar");
            }
            tracing::info!(
                version = env!("CARGO_PKG_VERSION"),
                login_item = ?layout_macos::login_item::status(),
                "IYAGI Layout started"
            );
            Ok(())
        })
        .on_window_event(|window, event| {
            // 설정 창 닫기 = 숨기기. 앱은 메뉴 막대에 남는다.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building IYAGI Layout")
        .run(|_app, event| {
            // 창이 하나도 없어도(또는 마지막 창이 닫혀도) 이벤트 루프를 유지한다.
            // 트레이 "종료"는 `app.exit(0)` → `code: Some(0)`이라 통과한다.
            if let tauri::RunEvent::ExitRequested {
                api, code: None, ..
            } = event
            {
                api.prevent_exit();
            }
        });
}
