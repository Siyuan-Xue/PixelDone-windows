use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::{
    Data::Xml::Dom::XmlDocument,
    Foundation::TypedEventHandler,
    UI::Notifications::{ToastActivatedEventArgs, ToastNotification, ToastNotificationManager},
    core::{HSTRING, Interface},
};

use crate::{
    application::{
        commands::reminder::{snooze_reminder, stop_reminder},
        state::ManagedAppState,
    },
    domain::AppError,
};

pub fn show_toast(
    app: &AppHandle,
    todo_ids: &[String],
    title: &str,
    body: &str,
) -> Result<(), AppError> {
    let xml = XmlDocument::new().map_err(platform_error)?;
    let payload = toast_xml(title, body);
    xml.LoadXml(&HSTRING::from(payload))
        .map_err(platform_error)?;
    let toast = ToastNotification::CreateToastNotification(&xml).map_err(platform_error)?;
    register_action_handler(&toast, app, todo_ids)?;
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(
        "com.milesxue.pixeldone.windows",
    ))
    .map_err(platform_error)?;
    notifier.Show(&toast).map_err(platform_error)
}

fn toast_xml(title: &str, body: &str) -> String {
    format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual><actions><action content=\"STOP\" arguments=\"stop\" activationType=\"foreground\"/><action content=\"SNOOZE 10 MIN\" arguments=\"snooze\" activationType=\"foreground\"/></actions><audio src=\"ms-winsoundevent:Notification.Default\"/></toast>",
        escape_xml(title),
        escape_xml(body),
    )
}

fn register_action_handler(
    toast: &ToastNotification,
    app: &AppHandle,
    todo_ids: &[String],
) -> Result<(), AppError> {
    let app = app.clone();
    let todo_ids = todo_ids.to_vec();
    toast
        .Activated(&TypedEventHandler::<
            ToastNotification,
            windows::core::IInspectable,
        >::new(move |_, args| {
            let action = args
                .as_ref()
                .and_then(|value| value.cast::<ToastActivatedEventArgs>().ok())
                .and_then(|value| value.Arguments().ok())
                .map(|value| value.to_string())
                .unwrap_or_default();
            if action != "stop" && action != "snooze" {
                return Ok(());
            }
            let app = app.clone();
            let todo_ids = todo_ids.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<ManagedAppState>();
                let revision = state.inner.lock().await.snapshot.revision;
                if action == "snooze" {
                    let _ = snooze_reminder(app.clone(), state, revision, todo_ids).await;
                } else {
                    let _ = stop_reminder(app.clone(), state, revision, todo_ids).await;
                }
            });
            Ok(())
        }))
        .map(|_| ())
        .map_err(platform_error)
}

pub fn show_xhigh_window(app: &AppHandle, todo_ids: &[String]) -> Result<(), AppError> {
    let query = todo_ids.join(",");
    if let Some(window) = app.get_webview_window("xhigh-alarm") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.set_always_on_top(true);
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "xhigh-alarm",
        WebviewUrl::App(format!("alarm?todoIds={query}").into()),
    )
    .title("PixelDone XHIGH")
    .inner_size(520.0, 360.0)
    .min_inner_size(480.0, 320.0)
    .center()
    .always_on_top(true)
    .focused(true)
    .build()
    .map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn platform_error(error: windows::core::Error) -> AppError {
    AppError::Platform(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_xml_escapes_user_titles() {
        assert_eq!(escape_xml("A & <B>"), "A &amp; &lt;B&gt;");
        let xml = toast_xml("A & <B>", "task");
        assert!(xml.contains("arguments=\"stop\""));
        assert!(xml.contains("arguments=\"snooze\""));
    }
}
