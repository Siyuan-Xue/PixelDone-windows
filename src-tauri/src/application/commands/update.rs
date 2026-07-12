use std::{
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use tauri::{Emitter, Manager, State};
use tauri_plugin_updater::UpdaterExt;

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, SnapshotDelta, UpdateView, unix_now_millis},
};

const SUCCESS_INTERVAL_MILLIS: i64 = 24 * 60 * 60 * 1_000;
const FAILURE_INTERVAL_MILLIS: i64 = 6 * 60 * 60 * 1_000;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateProgress {
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
}

pub fn start_automatic_update_checks(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        loop {
            automatic_check(&app).await;
            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
        }
    });
}

async fn automatic_check(app: &tauri::AppHandle) {
    let state = app.state::<ManagedAppState>();
    let now = unix_now_millis();
    {
        let runtime = state.inner.lock().await;
        if !runtime.snapshot.settings.automatic_update_check_enabled
            || runtime
                .snapshot
                .update
                .next_check_at_millis
                .is_some_and(|next| next > now)
        {
            return;
        }
    }
    let checked = perform_check(app).await;
    let next = now
        + if checked.is_ok() {
            SUCCESS_INTERVAL_MILLIS
        } else {
            FAILURE_INTERVAL_MILLIS
        };
    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    runtime.snapshot.update = checked.unwrap_or_else(|error| UpdateView {
        state: "ERROR".to_owned(),
        current_version: env!("CARGO_PKG_VERSION").to_owned(),
        message: Some(error.to_string()),
        ..UpdateView::default()
    });
    runtime.snapshot.update.last_checked_at_millis = Some(now);
    runtime.snapshot.update.next_check_at_millis = Some(next);
    runtime.snapshot.revision += 1;
    if runtime
        .repository
        .save_snapshot(&runtime.snapshot)
        .await
        .is_ok()
        && runtime
            .repository
            .save_update_timing(now, next)
            .await
            .is_ok()
    {
        let result = MutationResult {
            revision: runtime.snapshot.revision,
            changed_ids: vec!["update".to_owned()],
            snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
        };
        let _ = app.emit("snapshot://delta", result);
    }
}

#[tauri::command]
pub async fn check_for_update(
    app: tauri::AppHandle,
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    let now = unix_now_millis();
    let next = now + SUCCESS_INTERVAL_MILLIS;
    let mut view = perform_check(&app).await?;
    view.last_checked_at_millis = Some(now);
    view.next_check_at_millis = Some(next);
    let repository = state.inner.lock().await.repository.clone();
    repository.save_update_timing(now, next).await?;
    mutate(state, expected_revision, move |snapshot| {
        snapshot.update = view;
        Ok(vec!["update".to_owned()])
    })
    .await
}

async fn perform_check(app: &tauri::AppHandle) -> Result<UpdateView, AppError> {
    let update = app
        .updater()
        .map_err(|error| AppError::Update(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::Update(error.to_string()))?;
    Ok(if let Some(update) = update {
        UpdateView {
            state: "AVAILABLE".to_owned(),
            current_version: env!("CARGO_PKG_VERSION").to_owned(),
            available_version: Some(update.version.to_string()),
            source: Some("GitHub / Gitee signed updater manifest".to_owned()),
            message: Some(
                update
                    .body
                    .unwrap_or_else(|| "A new version is available".to_owned()),
            ),
            ..UpdateView::default()
        }
    } else {
        UpdateView {
            state: "CURRENT".to_owned(),
            current_version: env!("CARGO_PKG_VERSION").to_owned(),
            source: Some("GitHub / Gitee signed updater manifest".to_owned()),
            message: Some("PixelDone is up to date".to_owned()),
            ..UpdateView::default()
        }
    })
}

#[tauri::command]
pub async fn download_and_install_update(app: tauri::AppHandle) -> Result<(), AppError> {
    let update = app
        .updater()
        .map_err(|error| AppError::Update(error.to_string()))?
        .check()
        .await
        .map_err(|error| AppError::Update(error.to_string()))?
        .ok_or_else(|| AppError::Update("No update is available to install".to_owned()))?;
    let downloaded = Arc::new(AtomicU64::new(0));
    let progress_downloaded = downloaded.clone();
    let progress_app = app.clone();
    update
        .download_and_install(
            move |chunk, total| {
                let progress_app = progress_app.clone();
                let progress_downloaded = progress_downloaded.clone();
                let value =
                    progress_downloaded.fetch_add(chunk as u64, Ordering::Relaxed) + chunk as u64;
                let _ = progress_app.emit(
                    "update://progress",
                    UpdateProgress {
                        downloaded_bytes: value,
                        total_bytes: total,
                    },
                );
            },
            || {},
        )
        .await
        .map_err(|error| AppError::Update(error.to_string()))
}
