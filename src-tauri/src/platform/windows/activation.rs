//! Notification protocol activation routing for the installed Windows app.

use tauri::Manager;

use crate::{
    application::{
        commands::reminder::{snooze_reminder, stop_reminder},
        state::ManagedAppState,
    },
    domain::AppError,
};

pub fn route_arguments(app: &tauri::AppHandle, arguments: impl IntoIterator<Item = String>) {
    for argument in arguments {
        let Ok(url) = url::Url::parse(&argument) else {
            continue;
        };
        if url.scheme() != "pixeldone-reminder" {
            continue;
        }
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let _ = handle_url(&app, &url).await;
        });
    }
}

async fn handle_url(app: &tauri::AppHandle, url: &url::Url) -> Result<(), AppError> {
    let action = url.host_str().unwrap_or("open");
    let todo_id = url
        .query_pairs()
        .find(|(key, _)| key == "todo")
        .map(|(_, value)| value.into_owned())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AppError::Validation("Reminder activation is missing a task id".to_owned())
        })?;
    let state = app.state::<ManagedAppState>();
    let exists = state
        .inner
        .lock()
        .await
        .snapshot
        .checklists
        .iter()
        .flat_map(|list| list.items.iter())
        .any(|item| item.id == todo_id && !item.is_trashed());
    if !exists {
        return Err(AppError::NotFound("reminder task".to_owned()));
    }
    let revision = state.inner.lock().await.snapshot.revision;
    match action {
        "stop" => {
            stop_reminder(state, revision, vec![todo_id]).await?;
        }
        "snooze" => {
            snooze_reminder(state, revision, vec![todo_id]).await?;
        }
        "open" => show_main_window(app),
        _ => return Err(AppError::Validation("Unknown reminder action".to_owned())),
    }
    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reminder_protocol_is_strictly_scoped() {
        let url = url::Url::parse("pixeldone-reminder://snooze?todo=123").unwrap();
        assert_eq!(url.scheme(), "pixeldone-reminder");
        assert_eq!(url.host_str(), Some("snooze"));
        assert_eq!(url.query_pairs().next().unwrap().1, "123");
    }
}
