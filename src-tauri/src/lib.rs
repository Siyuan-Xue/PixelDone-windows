pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod platform;

use application::{
    commands,
    commands::update::start_automatic_update_checks,
    services::start_background_services,
    state::{AppPaths, ManagedAppState},
};
use domain::{AppError, AppSnapshot, SyncRunView, SyncState, unix_now_millis};
use infrastructure::{auth::SupabaseClient, repository::SqliteRepository};
use platform::windows::activation::route_arguments;
use platform::windows::credentials::CredentialStore;
use platform::windows::identity::ensure_notification_identity;
use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_autostart::ManagerExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let notification_identity_error = std::env::current_exe()
        .map_err(|error| AppError::Platform(error.to_string()))
        .and_then(|path| ensure_notification_identity(&path))
        .err()
        .map(|error| error.to_string());
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            route_arguments(app, args);
            show_main_window(app);
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--minimized"])
                .build(),
        );

    // WebDriver is deliberately limited to debug builds. The signed updater
    // artifact is a release build and therefore contains no automation server.
    #[cfg(debug_assertions)]
    let builder = builder
        .plugin(tauri_plugin_wdio::init())
        .plugin(tauri_plugin_wdio_webdriver::init());

    builder
        .setup(move |app| {
            #[cfg(debug_assertions)]
            let root = std::env::var_os("PIXELDONE_DATA_ROOT")
                .map(std::path::PathBuf::from)
                .unwrap_or(app.path().app_local_data_dir()?);
            #[cfg(not(debug_assertions))]
            let root = app.path().app_local_data_dir()?;
            let paths = AppPaths {
                data: root.join("data"),
                attachments: root.join("attachments"),
                cache: root.join("cache"),
                logs: root.join("logs"),
                webview_data: root.join("EBWebView"),
                legacy_roaming_database: app.path().app_data_dir()?.join("pixeldone.sqlite3"),
                root,
            };
            for path in [
                &paths.root,
                &paths.data,
                &paths.attachments,
                &paths.cache,
                &paths.logs,
            ] {
                std::fs::create_dir_all(path)?;
            }
            let database_path = paths.data.join("pixeldone.sqlite3");
            if !database_path.exists() && paths.legacy_roaming_database.is_file() {
                std::fs::copy(&paths.legacy_roaming_database, &database_path)?;
            }
            let repository = tauri::async_runtime::block_on(SqliteRepository::open(&database_path))
                .map_err(boxed_error)?;
            let mut snapshot = tauri::async_runtime::block_on(repository.load_snapshot())
                .map_err(boxed_error)?
                .unwrap_or_else(|| AppSnapshot::initial(unix_now_millis()));
            let cloud = SupabaseClient::from_build_config().map_err(boxed_error)?;
            let credentials = CredentialStore::default();
            #[cfg(debug_assertions)]
            if std::env::var_os("PIXELDONE_CLEAR_CREDENTIALS_ON_START").is_some() {
                credentials.clear().map_err(boxed_error)?;
            }
            let session = credentials.load().map_err(boxed_error)?;
            snapshot.auth = cloud.auth_view(session.as_ref());
            snapshot.sync = SyncRunView {
                state: if session.is_some() {
                    SyncState::Idle
                } else {
                    SyncState::SignedOut
                },
                message: Some(if session.is_some() {
                    "账号会话已从 Windows Credential Manager 恢复".to_owned()
                } else {
                    "登录后可与 Android 同步".to_owned()
                }),
                insecure_http: true,
                ..SyncRunView::default()
            };
            if let Some(error) = &notification_identity_error {
                snapshot.reminder.state = "IDENTITY_ERROR".to_owned();
                snapshot.reminder.message = Some(error.clone());
            }
            tauri::async_runtime::block_on(repository.save_snapshot(&snapshot))
                .map_err(boxed_error)?;
            let autostart_initialized =
                tauri::async_runtime::block_on(repository.autostart_initialized())
                    .map_err(boxed_error)?;
            if !cfg!(debug_assertions) && !autostart_initialized {
                if snapshot.settings.autostart_enabled {
                    app.autolaunch().enable()?;
                }
                tauri::async_runtime::block_on(repository.mark_autostart_initialized())
                    .map_err(boxed_error)?;
            }
            app.manage(ManagedAppState::new(
                snapshot,
                repository,
                cloud,
                credentials,
                session,
                paths,
                notification_identity_error.clone(),
            ));

            route_arguments(app.handle(), std::env::args());

            let show = MenuItem::with_id(app, "show", "打开 PixelDone", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quit])?;
            let mut tray = TrayIconBuilder::with_id("main")
                .menu(&menu)
                .tooltip("PixelDone")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "show" => show_main_window(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main_window(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;

            if std::env::args().any(|argument| argument == "--minimized")
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.hide();
            }
            start_background_services(app.handle().clone());
            start_automatic_update_checks(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::select_checklist,
            commands::back_checklist,
            commands::create_checklist,
            commands::rename_checklist,
            commands::delete_checklist,
            commands::create_todo,
            commands::update_todo,
            commands::toggle_todo,
            commands::move_todo_to_trash,
            commands::clean_completed,
            commands::restore_todo,
            commands::purge_todo,
            commands::purge_all_trash,
            commands::set_sort_mode,
            commands::set_hide_completed,
            commands::set_quick_delete,
            commands::set_deadline_countdown,
            commands::update_settings,
            commands::auth::auth_sign_in,
            commands::auth::auth_sign_up,
            commands::auth::auth_sign_out,
            commands::auth::auth_change_password,
            commands::sync::sync_now,
            commands::sync::load_sync_conflicts,
            commands::sync::resolve_sync_conflict,
            commands::image::attach_todo_image,
            commands::image::delete_todo_image,
            commands::image::load_todo_image_preview,
            commands::reminder::stop_reminder,
            commands::reminder::snooze_reminder,
            commands::storage::get_storage_info,
            commands::storage::open_data_folder,
            commands::storage::delete_legacy_roaming_data,
            commands::update::check_for_update,
            commands::update::download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PixelDone");
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn boxed_error(error: AppError) -> Box<dyn std::error::Error> {
    Box::new(error)
}
