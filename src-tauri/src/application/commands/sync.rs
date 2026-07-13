use tauri::State;

use crate::{
    application::state::ManagedAppState,
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
    let _gate = state.sync_gate.lock().await;
    let session = state
        .session
        .lock()
        .await
        .clone()
        .ok_or_else(|| AppError::Auth("请先登录".to_owned()))?;
    let session = state.cloud.refresh_if_needed(&session, false).await?;
    state.credentials.save(&session)?;
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
            runtime.snapshot.sync = SyncRunView {
                state: if error.to_string().contains("SERVER UPDATE REQUIRED") {
                    SyncState::ServerUpdateRequired
                } else {
                    SyncState::Error
                },
                message: Some(error.to_string()),
                insecure_http: true,
                ..runtime.snapshot.sync.clone()
            };
            repository.save_snapshot(&runtime.snapshot).await?;
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
    Ok(MutationResult {
        revision: runtime.snapshot.revision,
        changed_ids: vec![record_type, local_id],
        snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
    })
}
