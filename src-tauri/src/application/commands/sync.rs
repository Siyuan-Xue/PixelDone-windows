use tauri::State;

use crate::{
    application::{commands::ensure_revision, state::ManagedAppState},
    domain::{
        AppError, ConflictResolutionChoice, MutationResult, SnapshotDelta, SyncConflictView,
        SyncRunView, SyncState,
    },
    infrastructure::sync::{conflicts, resolve_conflict, run_sync},
};

#[tauri::command]
pub async fn sync_now(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    ensure_revision(&state, expected_revision).await?;
    let _gate = state.sync_gate.lock().await;
    let session = state
        .session
        .lock()
        .await
        .clone()
        .ok_or_else(|| AppError::Auth("请先登录".to_owned()))?;
    let session = match state.cloud.refresh_if_needed(&session, false).await {
        Ok(session) => session,
        Err(error) => return record_early_sync_failure(&state, error).await,
    };
    if let Err(error) = state.credentials.save(&session) {
        return record_early_sync_failure(&state, error).await;
    }
    *state.session.lock().await = Some(session.clone());

    let mut runtime = state.inner.lock().await;
    if runtime.snapshot.revision != expected_revision {
        return Err(AppError::StaleRevision);
    }
    let before = runtime.snapshot.clone();
    let repository = runtime.repository.clone();
    runtime.snapshot.sync = SyncRunView {
        state: SyncState::Syncing,
        message: Some("正在同步".to_owned()),
        issue_code: None,
        next_retry_at_millis: None,
        insecure_http: true,
        ..runtime.snapshot.sync.clone()
    };
    let result = run_sync(
        &state.cloud,
        &repository,
        &session,
        &mut runtime.snapshot,
        &state.paths.attachments,
    )
    .await;
    runtime.snapshot.revision += 1;
    match result {
        Ok(view) => runtime.snapshot.sync = view,
        Err(error) => {
            let retryable = error.is_retryable_sync_error();
            let auth_expired = matches!(&error, AppError::Auth(_));
            runtime.snapshot.sync = SyncRunView {
                state: if matches!(&error, AppError::ServerUpdateRequired(_)) {
                    SyncState::ServerUpdateRequired
                } else {
                    SyncState::Error
                },
                message: Some(error.to_string()),
                issue_code: Some(error.sync_issue_code()),
                next_retry_at_millis: retryable.then(|| crate::domain::unix_now_millis() + 1_000),
                insecure_http: true,
                ..runtime.snapshot.sync.clone()
            };
            if auth_expired {
                runtime.snapshot.auth = state.cloud.auth_view(None);
            }
            repository.save_snapshot(&runtime.snapshot).await?;
            drop(runtime);
            if auth_expired {
                let _ = state.credentials.clear();
                *state.session.lock().await = None;
                state.auth_notify.notify_one();
            }
            if retryable {
                state.sync_notify.notify_one();
            }
            state.reminder_notify.notify_one();
            return Err(error);
        }
    }
    runtime.snapshot.auth = state.cloud.auth_view(Some(&session));
    repository.save_snapshot(&runtime.snapshot).await?;
    let snapshot_delta = SnapshotDelta::between(&before, &runtime.snapshot);
    state.reminder_notify.notify_one();
    Ok(MutationResult {
        revision: runtime.snapshot.revision,
        changed_ids: vec!["sync".to_owned()],
        snapshot_delta,
    })
}

async fn record_early_sync_failure(
    state: &State<'_, ManagedAppState>,
    error: AppError,
) -> Result<MutationResult, AppError> {
    let retryable = error.is_retryable_sync_error();
    let auth_expired = matches!(&error, AppError::Auth(_));
    if auth_expired {
        let _ = state.credentials.clear();
        *state.session.lock().await = None;
        state.auth_notify.notify_one();
    }
    let mut runtime = state.inner.lock().await;
    if auth_expired {
        runtime.snapshot.auth = state.cloud.auth_view(None);
    }
    runtime.snapshot.sync = SyncRunView {
        state: if matches!(&error, AppError::ServerUpdateRequired(_)) {
            SyncState::ServerUpdateRequired
        } else {
            SyncState::Error
        },
        message: Some(error.to_string()),
        issue_code: Some(error.sync_issue_code()),
        next_retry_at_millis: retryable.then(|| crate::domain::unix_now_millis() + 1_000),
        insecure_http: true,
        ..runtime.snapshot.sync.clone()
    };
    runtime.snapshot.revision += 1;
    runtime.repository.save_snapshot(&runtime.snapshot).await?;
    drop(runtime);
    if retryable {
        state.sync_notify.notify_one();
    }
    state.reminder_notify.notify_one();
    Err(error)
}

#[tauri::command]
pub async fn load_sync_conflicts(
    state: State<'_, ManagedAppState>,
) -> Result<Vec<SyncConflictView>, AppError> {
    let runtime = state.inner.lock().await;
    conflicts(&runtime.repository).await
}

#[tauri::command]
pub async fn resolve_sync_conflict(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    record_type: String,
    local_id: String,
    choice: ConflictResolutionChoice,
) -> Result<MutationResult, AppError> {
    let mut runtime = state.inner.lock().await;
    if runtime.snapshot.revision != expected_revision {
        return Err(AppError::StaleRevision);
    }
    let before = runtime.snapshot.clone();
    let repository = runtime.repository.clone();
    resolve_conflict(
        &repository,
        &mut runtime.snapshot,
        &record_type,
        &local_id,
        choice == ConflictResolutionChoice::KeepCloud,
    )
    .await?;
    runtime.snapshot.revision += 1;
    let conflict_count = repository.conflicts().await?.len();
    runtime.snapshot.sync.conflict_count = conflict_count;
    runtime.snapshot.sync.state = if conflict_count == 0 {
        SyncState::Idle
    } else {
        SyncState::Conflict
    };
    repository.save_snapshot(&runtime.snapshot).await?;
    let result = MutationResult {
        revision: runtime.snapshot.revision,
        changed_ids: vec![record_type, local_id],
        snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
    };
    drop(runtime);
    state.sync_notify.notify_one();
    state.reminder_notify.notify_one();
    Ok(result)
}
