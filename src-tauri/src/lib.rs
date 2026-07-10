pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod platform;

use application::{commands, state::ManagedAppState};
use domain::{AppError, AppSnapshot, unix_now_millis};
use infrastructure::repository::SqliteRepository;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let database_path = data_dir.join("pixeldone.sqlite3");
            let repository = tauri::async_runtime::block_on(SqliteRepository::open(&database_path))
                .map_err(boxed_error)?;
            let snapshot = tauri::async_runtime::block_on(repository.load_snapshot())
                .map_err(boxed_error)?
                .unwrap_or_else(|| AppSnapshot::initial(unix_now_millis()));
            tauri::async_runtime::block_on(repository.save_snapshot(&snapshot))
                .map_err(boxed_error)?;
            app.manage(ManagedAppState::new(snapshot, repository));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::select_checklist,
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
            commands::set_sort_mode,
            commands::set_hide_completed,
            commands::set_quick_delete,
            commands::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PixelDone");
}

fn boxed_error(error: AppError) -> Box<dyn std::error::Error> {
    Box::new(error)
}
