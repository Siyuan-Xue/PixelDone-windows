use std::path::Path;

use tauri::State;
use tauri_plugin_opener::OpenerExt;

use crate::{
    application::state::ManagedAppState,
    domain::{AppError, StorageInfo},
};

const CREDENTIAL_TARGET: &str = "com.milesxue.pixeldone.windows/supabase-session";

#[tauri::command]
pub async fn get_storage_info(state: State<'_, ManagedAppState>) -> Result<StorageInfo, AppError> {
    let executable =
        std::env::current_exe().map_err(|error| AppError::Platform(error.to_string()))?;
    let legacy = &state.paths.legacy_roaming_database;
    let legacy_exists = legacy.is_file();
    Ok(StorageInfo {
        executable_path: executable.display().to_string(),
        install_directory: executable
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .display()
            .to_string(),
        data_root: state.paths.root.display().to_string(),
        database_path: state
            .paths
            .data
            .join("pixeldone.sqlite3")
            .display()
            .to_string(),
        attachments_path: state.paths.attachments.display().to_string(),
        cache_path: state.paths.cache.display().to_string(),
        logs_path: state.paths.logs.display().to_string(),
        webview_data_path: state.paths.webview_data.display().to_string(),
        total_bytes: directory_size(&state.paths.root),
        legacy_roaming_database_path: legacy_exists.then(|| legacy.display().to_string()),
        legacy_roaming_database_bytes: legacy_exists.then(|| {
            legacy
                .metadata()
                .map(|value| value.len())
                .unwrap_or_default()
        }),
        credential_manager_target: CREDENTIAL_TARGET.to_owned(),
    })
}

#[tauri::command]
pub async fn open_data_folder(
    app: tauri::AppHandle,
    state: State<'_, ManagedAppState>,
) -> Result<(), AppError> {
    app.opener()
        .open_path(state.paths.root.display().to_string(), None::<&str>)
        .map_err(|error| AppError::Platform(error.to_string()))
}

#[tauri::command]
pub async fn delete_legacy_roaming_data(
    state: State<'_, ManagedAppState>,
    confirmed: bool,
) -> Result<(), AppError> {
    if !confirmed {
        return Err(AppError::Validation(
            "Deletion requires explicit confirmation".to_owned(),
        ));
    }
    if state.paths.legacy_roaming_database.is_file() {
        std::fs::remove_file(&state.paths.legacy_roaming_database)
            .map_err(|error| AppError::Platform(error.to_string()))?;
    }
    Ok(())
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                directory_size(&path)
            } else {
                entry
                    .metadata()
                    .map(|value| value.len())
                    .unwrap_or_default()
            }
        })
        .sum()
}
