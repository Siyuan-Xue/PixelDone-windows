use tauri::State;

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, unix_now_millis},
    infrastructure::reminder::ReminderOccurrence,
};

#[tauri::command]
pub async fn stop_reminder(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    todo_ids: Vec<String>,
) -> Result<MutationResult, AppError> {
    let repository = state.inner.lock().await.repository.clone();
    let fired_at = unix_now_millis();
    for id in &todo_ids {
        repository
            .save_reminder_delivery_state(id, None, None, Some(fired_at))
            .await?;
    }
    mutate(state, expected_revision, |snapshot| {
        snapshot.reminder.state = "IDLE".to_owned();
        snapshot.reminder.active_todo_ids.clear();
        snapshot.reminder.last_fired_at_millis = Some(fired_at);
        Ok(todo_ids)
    })
    .await
}

#[tauri::command]
pub async fn snooze_reminder(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    todo_ids: Vec<String>,
) -> Result<MutationResult, AppError> {
    let until = unix_now_millis() + crate::domain::reminder::DEFAULT_SNOOZE_MILLIS;
    let (repository, reminders) = {
        let runtime = state.inner.lock().await;
        let reminders = todo_ids
            .iter()
            .filter_map(|id| {
                runtime
                    .snapshot
                    .checklists
                    .iter()
                    .flat_map(|list| list.items.iter())
                    .find(|item| &item.id == id)
                    .map(|item| ReminderOccurrence {
                        todo_id: item.id.clone(),
                        title: item.title.clone(),
                        priority: item.priority,
                        delivery_at_millis: until,
                        enhanced_alarm: runtime.snapshot.settings.enhanced_xhigh_alarm_enabled
                            && item.priority == crate::domain::TodoPriority::Xhigh,
                    })
            })
            .collect::<Vec<_>>();
        (runtime.repository.clone(), reminders)
    };
    for reminder in &reminders {
        repository
            .save_reminder_delivery_state(
                &reminder.todo_id,
                Some(until),
                Some(until),
                Some(until - crate::domain::reminder::DEFAULT_SNOOZE_MILLIS),
            )
            .await?;
    }
    mutate(state, expected_revision, |snapshot| {
        snapshot.reminder.state = "SNOOZED".to_owned();
        snapshot.reminder.active_todo_ids.clear();
        snapshot.reminder.last_fired_at_millis =
            Some(until - crate::domain::reminder::DEFAULT_SNOOZE_MILLIS);
        Ok(todo_ids)
    })
    .await
}
