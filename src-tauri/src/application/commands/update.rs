use tauri::State;
use tauri_plugin_updater::UpdaterExt;

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, UpdateView},
};

#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    let update = app
        .updater()
        .map_err(|error| AppError::Update(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::Update(error.to_string()))?;
    let view = if let Some(update) = update {
        UpdateView {
            state: "AVAILABLE".to_owned(),
            current_version: env!("CARGO_PKG_VERSION").to_owned(),
            available_version: Some(update.version.to_string()),
            download_url: None,
            source: Some("GitHub / Gitee updater manifest".to_owned()),
            message: Some(update.body.unwrap_or_else(|| "发现新版本".to_owned())),
        }
    } else {
        UpdateView {
            state: "CURRENT".to_owned(),
            current_version: env!("CARGO_PKG_VERSION").to_owned(),
            available_version: None,
            download_url: None,
            source: Some("GitHub / Gitee updater manifest".to_owned()),
            message: Some("当前已经是最新正式版".to_owned()),
        }
    };
    mutate(state, expected_revision, move |snapshot| {
        snapshot.update = view;
        Ok(vec!["update".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn download_and_install_update(app: tauri::AppHandle) -> Result<(), AppError> {
    let update = app
        .updater()
        .map_err(|error| AppError::Update(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::Update(error.to_string()))?
        .ok_or_else(|| AppError::Update("没有可安装的更新".to_owned()))?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|error| AppError::Update(error.to_string()))
}
