use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::{
    application::state::ManagedAppState,
    domain::{MutationResult, ReminderRepeat, SnapshotDelta, TodoPriority, unix_now_millis},
    platform::windows::notification::{show_toast, show_xhigh_window},
};

pub fn start_background_services(app: AppHandle) {
    let reminder_app = app.clone();
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            reconcile_reminders(&reminder_app).await;
        }
    });
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            let state = app.state::<ManagedAppState>();
            tokio::select! {
                () = state.sync_notify.notified() => {
                    tokio::time::sleep(Duration::from_millis(700)).await;
                }
                _ = interval.tick() => {}
            }
            automatic_sync(&app).await;
        }
    });
}

async fn automatic_sync(app: &AppHandle) {
    use crate::{
        domain::{SyncRunView, SyncState},
        infrastructure::sync::run_sync,
    };

    let state = app.state::<ManagedAppState>();
    let Some(session) = state.session.lock().await.clone() else {
        return;
    };
    let Ok(_gate) = state.sync_gate.try_lock() else {
        return;
    };
    let session = match state.cloud.refresh_if_needed(&session, false).await {
        Ok(session) => session,
        Err(_) => return,
    };
    if state.credentials.save(&session).is_err() {
        return;
    }
    *state.session.lock().await = Some(session.clone());
    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    let repository = runtime.repository.clone();
    match run_sync(&state.cloud, &repository, &session, &mut runtime.snapshot).await {
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
        }
    }
    runtime.snapshot.auth = state.cloud.auth_view(Some(&session));
    if runtime.snapshot != before {
        runtime.snapshot.revision += 1;
        if repository.save_snapshot(&runtime.snapshot).await.is_ok() {
            let result = MutationResult {
                revision: runtime.snapshot.revision,
                changed_ids: vec!["sync".to_owned()],
                snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
            };
            let _ = app.emit("snapshot://delta", result);
            let _ = app.emit("sync://state", runtime.snapshot.sync.clone());
        }
    }
}

async fn reconcile_reminders(app: &AppHandle) {
    let state = app.state::<ManagedAppState>();
    let now = unix_now_millis();
    let snoozed = state.snoozed_until.lock().await.clone();
    let fired = state.fired_reminders.lock().await.clone();
    let due = {
        let runtime = state.inner.lock().await;
        runtime
            .snapshot
            .checklists
            .iter()
            .filter(|list| list.kind == crate::domain::ChecklistKind::Normal)
            .flat_map(|list| list.items.iter())
            .filter(|item| !item.completed && !item.is_trashed())
            .filter(|item| {
                let effective_due = snoozed.get(&item.id).copied().unwrap_or(item.due_at_millis);
                effective_due > 0
                    && effective_due <= now
                    && fired.get(&item.id).copied() != Some(effective_due)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    if due.is_empty() {
        return;
    }

    {
        let mut fired = state.fired_reminders.lock().await;
        for item in &due {
            let effective_due = snoozed.get(&item.id).copied().unwrap_or(item.due_at_millis);
            fired.insert(item.id.clone(), effective_due);
        }
    }
    {
        let mut snoozed = state.snoozed_until.lock().await;
        for item in &due {
            snoozed.remove(&item.id);
        }
    }

    let xhigh = due
        .iter()
        .filter(|item| item.priority == TodoPriority::Xhigh)
        .map(|item| item.id.clone())
        .collect::<Vec<_>>();
    let normal = due
        .iter()
        .filter(|item| item.priority != TodoPriority::Xhigh)
        .collect::<Vec<_>>();
    if !normal.is_empty() {
        let body = normal
            .iter()
            .take(3)
            .map(|item| item.title.as_str())
            .collect::<Vec<_>>()
            .join(" · ");
        let ids = normal
            .iter()
            .map(|item| item.id.clone())
            .collect::<Vec<_>>();
        let _ = show_toast(app, &ids, "PixelDone", &body);
    }
    if !xhigh.is_empty() {
        let _ = show_xhigh_window(app, &xhigh);
    }

    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    for list in &mut runtime.snapshot.checklists {
        for item in &mut list.items {
            if !due.iter().any(|fired| fired.id == item.id) {
                continue;
            }
            let interval = match item.reminder_repeat {
                ReminderRepeat::None => None,
                ReminderRepeat::Daily => Some(24 * 60 * 60 * 1_000),
                ReminderRepeat::Weekly => Some(7 * 24 * 60 * 60 * 1_000),
            };
            if let Some(interval) = interval {
                while item.due_at_millis <= now {
                    item.due_at_millis += interval;
                }
                item.updated_at_millis = now;
            }
        }
    }
    runtime.snapshot.reminder.state = if xhigh.is_empty() {
        "FIRED".to_owned()
    } else {
        "XHIGH".to_owned()
    };
    runtime.snapshot.reminder.active_todo_ids = due.iter().map(|item| item.id.clone()).collect();
    runtime.snapshot.reminder.last_fired_at_millis = Some(now);
    runtime.snapshot.revision += 1;
    if runtime
        .repository
        .save_snapshot_with_changes(&before, &runtime.snapshot)
        .await
        .is_ok()
    {
        let result = MutationResult {
            revision: runtime.snapshot.revision,
            changed_ids: due.iter().map(|item| item.id.clone()).collect(),
            snapshot_delta: SnapshotDelta::between(&before, &runtime.snapshot),
        };
        let _ = app.emit("snapshot://delta", result);
        let _ = app.emit("reminder://fired", runtime.snapshot.reminder.clone());
    }
}
