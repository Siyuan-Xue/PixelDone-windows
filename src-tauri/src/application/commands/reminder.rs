use tauri::{Manager, State};

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, unix_now_millis},
};

#[tauri::command]
pub async fn stop_reminder(
    app: tauri::AppHandle,
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    todo_ids: Vec<String>,
) -> Result<MutationResult, AppError> {
    if let Some(window) = app.get_webview_window("xhigh-alarm") {
        let _ = window.close();
    }
    mutate(state, expected_revision, |snapshot| {
        snapshot.reminder.state = "IDLE".to_owned();
        snapshot.reminder.active_todo_ids.clear();
        Ok(todo_ids)
    })
    .await
}

#[tauri::command]
pub async fn snooze_reminder(
    app: tauri::AppHandle,
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    todo_ids: Vec<String>,
) -> Result<MutationResult, AppError> {
    let until = unix_now_millis() + crate::domain::reminder::DEFAULT_SNOOZE_MILLIS;
    {
        let mut snoozed = state.snoozed_until.lock().await;
        let mut fired = state.fired_reminders.lock().await;
        for id in &todo_ids {
            snoozed.insert(id.clone(), until);
            fired.remove(id);
        }
    }
    if let Some(window) = app.get_webview_window("xhigh-alarm") {
        let _ = window.close();
    }
    mutate(state, expected_revision, |snapshot| {
        snapshot.reminder.state = "SNOOZED".to_owned();
        snapshot.reminder.active_todo_ids.clear();
        Ok(todo_ids)
    })
    .await
}
