use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    application::state::ManagedAppState,
    domain::{MutationResult, SnapshotDelta, unix_now_millis},
    infrastructure::realtime::{ConnectionExit, listen_for_invalidations, retry_delay},
    infrastructure::reminder::{
        ReminderOccurrence, SCHEDULE_HORIZON_MILLIS, SCHEDULE_LIMIT, occurrences,
    },
    platform::windows::{
        identity::ensure_notification_identity,
        notification::{is_element_not_found_app_error, replace_scheduled_toasts},
    },
};

pub fn start_background_services(app: AppHandle) {
    let reminder_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15 * 60));
        loop {
            let state = reminder_app.state::<ManagedAppState>();
            tokio::select! {
                () = state.reminder_notify.notified() => {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
                _ = interval.tick() => {}
            }
            reconcile_system_reminders(&reminder_app).await;
        }
    });
    let sync_app = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            let state = sync_app.state::<ManagedAppState>();
            state.sync_notify.notified().await;
            let mut attempt = 0_u32;
            let mut first_run = true;
            loop {
                let delay = if first_run {
                    Duration::from_millis(700)
                } else {
                    retry_delay(attempt.saturating_sub(1))
                };
                tokio::select! {
                    () = state.sync_notify.notified() => {
                        attempt = 0;
                        first_run = true;
                        continue;
                    }
                    () = tokio::time::sleep(delay) => {}
                }
                let next_delay = retry_delay(attempt);
                let next_retry_at_millis =
                    unix_now_millis() + i64::try_from(next_delay.as_millis()).unwrap_or(30_000);
                match automatic_sync(&sync_app, next_retry_at_millis).await {
                    AutomaticSyncOutcome::Complete => break,
                    AutomaticSyncOutcome::Retry => {
                        attempt = attempt.saturating_add(1);
                        first_run = false;
                    }
                }
            }
        }
    });
    tauri::async_runtime::spawn(realtime_service(app));
}

async fn realtime_service(app: AppHandle) {
    let mut attempt = 0_u32;
    loop {
        let state = app.state::<ManagedAppState>();
        let Some(session) = state.session.lock().await.clone() else {
            state.auth_notify.notified().await;
            attempt = 0;
            continue;
        };
        let session = match state.cloud.refresh_if_needed(&session, false).await {
            Ok(session) => session,
            Err(_) => {
                let delay = retry_delay(attempt);
                attempt = attempt.saturating_add(1);
                tokio::select! {
                    () = state.auth_notify.notified() => attempt = 0,
                    () = tokio::time::sleep(delay) => {}
                }
                continue;
            }
        };
        if state.credentials.save(&session).is_err() {
            tokio::time::sleep(retry_delay(attempt)).await;
            attempt = attempt.saturating_add(1);
            continue;
        }
        *state.session.lock().await = Some(session.clone());
        match listen_for_invalidations(
            &state.cloud,
            &session,
            &state.sync_notify,
            &state.auth_notify,
        )
        .await
        {
            Ok(ConnectionExit::AuthenticationChanged | ConnectionExit::TokenRefreshRequired) => {
                attempt = 0;
            }
            Err(_) => {
                let delay = retry_delay(attempt);
                attempt = attempt.saturating_add(1);
                tokio::select! {
                    () = state.auth_notify.notified() => attempt = 0,
                    () = tokio::time::sleep(delay) => {}
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AutomaticSyncOutcome {
    Complete,
    Retry,
}

async fn automatic_sync(app: &AppHandle, next_retry_at_millis: i64) -> AutomaticSyncOutcome {
    use crate::{
        domain::{SyncRunView, SyncState},
        infrastructure::sync::run_sync,
    };

    let state = app.state::<ManagedAppState>();
    let Some(session) = state.session.lock().await.clone() else {
        return AutomaticSyncOutcome::Complete;
    };
    let Ok(_gate) = state.sync_gate.try_lock() else {
        return AutomaticSyncOutcome::Complete;
    };
    let session = match state.cloud.refresh_if_needed(&session, false).await {
        Ok(session) => session,
        Err(error) => {
            return publish_automatic_sync_failure(app, &state, error, next_retry_at_millis).await;
        }
    };
    if let Err(error) = state.credentials.save(&session) {
        return publish_automatic_sync_failure(app, &state, error, next_retry_at_millis).await;
    }
    *state.session.lock().await = Some(session.clone());
    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    let repository = runtime.repository.clone();
    let mut auth_expired = false;
    let outcome = match run_sync(
        &state.cloud,
        &repository,
        &session,
        &mut runtime.snapshot,
        &state.paths.attachments,
    )
    .await
    {
        Ok(view) => {
            runtime.snapshot.sync = view;
            AutomaticSyncOutcome::Complete
        }
        Err(error) => {
            let retryable = error.is_retryable_sync_error();
            auth_expired = matches!(&error, crate::domain::AppError::Auth(_));
            runtime.snapshot.sync = SyncRunView {
                state: if matches!(&error, crate::domain::AppError::ServerUpdateRequired(_)) {
                    SyncState::ServerUpdateRequired
                } else {
                    SyncState::Error
                },
                message: Some(error.to_string()),
                issue_code: Some(error.sync_issue_code()),
                next_retry_at_millis: retryable.then_some(next_retry_at_millis),
                insecure_http: true,
                ..runtime.snapshot.sync.clone()
            };
            if auth_expired {
                runtime.snapshot.auth = state.cloud.auth_view(None);
            }
            if retryable {
                AutomaticSyncOutcome::Retry
            } else {
                AutomaticSyncOutcome::Complete
            }
        }
    };
    if !auth_expired {
        runtime.snapshot.auth = state.cloud.auth_view(Some(&session));
    }
    if runtime.snapshot != before {
        runtime.snapshot.revision += 1;
        let _ = repository.save_snapshot(&runtime.snapshot).await;
        let result = MutationResult {
            revision: runtime.snapshot.revision,
            changed_ids: vec!["sync".to_owned()],
            snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
        };
        let _ = app.emit("snapshot://delta", result);
        let _ = app.emit("sync://state", runtime.snapshot.sync.clone());
    }
    drop(runtime);
    if auth_expired {
        let _ = state.credentials.clear();
        *state.session.lock().await = None;
        state.auth_notify.notify_one();
    }
    state.reminder_notify.notify_one();
    outcome
}

async fn publish_automatic_sync_failure(
    app: &AppHandle,
    state: &ManagedAppState,
    error: crate::domain::AppError,
    next_retry_at_millis: i64,
) -> AutomaticSyncOutcome {
    use crate::domain::{SyncRunView, SyncState};

    let retryable = error.is_retryable_sync_error();
    let auth_expired = matches!(&error, crate::domain::AppError::Auth(_));
    if auth_expired {
        let _ = state.credentials.clear();
        *state.session.lock().await = None;
        state.auth_notify.notify_one();
    }
    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    if auth_expired {
        runtime.snapshot.auth = state.cloud.auth_view(None);
    }
    runtime.snapshot.sync = SyncRunView {
        state: if matches!(&error, crate::domain::AppError::ServerUpdateRequired(_)) {
            SyncState::ServerUpdateRequired
        } else {
            SyncState::Error
        },
        message: Some(error.to_string()),
        issue_code: Some(error.sync_issue_code()),
        next_retry_at_millis: retryable.then_some(next_retry_at_millis),
        insecure_http: true,
        ..runtime.snapshot.sync.clone()
    };
    if runtime.snapshot != before {
        runtime.snapshot.revision += 1;
        let _ = runtime.repository.save_snapshot(&runtime.snapshot).await;
        let result = MutationResult {
            revision: runtime.snapshot.revision,
            changed_ids: vec!["sync".to_owned()],
            snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
        };
        let _ = app.emit("snapshot://delta", result);
        let _ = app.emit("sync://state", runtime.snapshot.sync.clone());
    }
    if retryable {
        AutomaticSyncOutcome::Retry
    } else {
        AutomaticSyncOutcome::Complete
    }
}

pub async fn reconcile_system_reminders(app: &AppHandle) {
    let state = app.state::<ManagedAppState>();
    let _gate = state.reminder_gate.lock().await;
    if let Some(error) = &state.notification_identity_error {
        publish_reminder_error(app, &state, "IDENTITY_ERROR", error.clone()).await;
        return;
    }
    let now = unix_now_millis();
    let (snapshot, repository) = {
        let runtime = state.inner.lock().await;
        (runtime.snapshot.clone(), runtime.repository.clone())
    };
    let (mut scheduled, mut truncated) = occurrences(&snapshot, now);
    if let Ok(snoozed) = repository.snoozed_reminders(now).await {
        for (todo_id, delivery_at_millis) in snoozed {
            let Some(item) = snapshot
                .checklists
                .iter()
                .flat_map(|list| list.items.iter())
                .find(|item| item.id == todo_id && !item.completed && !item.is_trashed())
            else {
                continue;
            };
            scheduled.push(ReminderOccurrence {
                todo_id,
                title: item.title.clone(),
                priority: item.priority,
                delivery_at_millis,
                enhanced_alarm: snapshot.settings.enhanced_xhigh_alarm_enabled
                    && item.priority == crate::domain::TodoPriority::Xhigh,
            });
        }
        scheduled.sort_by_key(|value| value.delivery_at_millis);
        truncated |= scheduled.len() > SCHEDULE_LIMIT;
        scheduled.truncate(SCHEDULE_LIMIT);
    }
    let mut result = replace_scheduled_toasts(&scheduled);
    if result.as_ref().is_err_and(is_element_not_found_app_error)
        && let Ok(executable) = std::env::current_exe()
        && ensure_notification_identity(&executable).is_ok()
    {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        result = replace_scheduled_toasts(&scheduled);
    }
    if result.is_ok() {
        let active_ids = snapshot
            .checklists
            .iter()
            .flat_map(|list| list.items.iter())
            .filter(|item| !item.completed && !item.is_trashed())
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();
        let _ = repository
            .delete_orphaned_reminder_delivery_states(&active_ids)
            .await;
        for todo_id in &active_ids {
            let next = scheduled
                .iter()
                .find(|value| &value.todo_id == todo_id)
                .map(|value| value.delivery_at_millis);
            if let Ok(previous) = repository.reminder_delivery_state(todo_id).await {
                let _ = repository
                    .save_reminder_delivery_state(
                        todo_id,
                        next,
                        previous.snoozed_until_millis.filter(|until| *until > now),
                        previous.last_fired_at_millis,
                    )
                    .await;
            }
        }
    }

    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    match result {
        Ok(count) => {
            runtime.snapshot.reminder.state = if truncated {
                "DEGRADED"
            } else if count == 0 {
                "IDLE"
            } else {
                "SCHEDULED"
            }
            .to_owned();
            runtime.snapshot.reminder.scheduled_count = count;
            runtime.snapshot.reminder.schedule_horizon_at_millis =
                Some(now + SCHEDULE_HORIZON_MILLIS);
            runtime.snapshot.reminder.schedule_truncated = truncated;
            runtime.snapshot.reminder.message = truncated
                .then(|| "Windows reminder queue reached the 4,000-item safety limit".to_owned());
        }
        Err(error) => {
            runtime.snapshot.reminder.state =
                if matches!(&error, crate::domain::AppError::NotificationsDisabled(_)) {
                    "DISABLED_BY_SYSTEM"
                } else {
                    "ERROR"
                }
                .to_owned();
            runtime.snapshot.reminder.message = Some(error.to_string());
        }
    }
    if runtime.snapshot.reminder == before.reminder {
        return;
    }
    runtime.snapshot.revision += 1;
    if runtime
        .repository
        .save_snapshot(&runtime.snapshot)
        .await
        .is_ok()
    {
        let result = MutationResult {
            revision: runtime.snapshot.revision,
            changed_ids: vec!["reminder-schedule".to_owned()],
            snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
        };
        let _ = app.emit("snapshot://delta", result);
        let _ = app.emit("reminder://state", runtime.snapshot.reminder.clone());
    }
}

async fn publish_reminder_error(
    app: &AppHandle,
    state: &ManagedAppState,
    status: &str,
    message: String,
) {
    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    runtime.snapshot.reminder.state = status.to_owned();
    runtime.snapshot.reminder.message = Some(message);
    if runtime.snapshot.reminder == before.reminder {
        return;
    }
    runtime.snapshot.revision += 1;
    if runtime
        .repository
        .save_snapshot(&runtime.snapshot)
        .await
        .is_ok()
    {
        let result = MutationResult {
            revision: runtime.snapshot.revision,
            changed_ids: vec!["reminder-schedule".to_owned()],
            snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
        };
        let _ = app.emit("snapshot://delta", result);
        let _ = app.emit("reminder://state", runtime.snapshot.reminder.clone());
    }
}
