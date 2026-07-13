use sha2::{Digest, Sha256};
use windows::{
    Data::Xml::Dom::XmlDocument,
    Foundation::DateTime,
    UI::Notifications::{
        NotificationSetting, ScheduledToastNotification, ToastNotificationManager, ToastNotifier,
    },
    core::HSTRING,
};

use crate::{domain::AppError, infrastructure::reminder::ReminderOccurrence};

pub const APP_USER_MODEL_ID: &str = "com.milesxue.pixeldone.windows";
const REMINDER_GROUP: &str = "pixeldone-reminders";
const WINDOWS_EPOCH_TICKS: i64 = 116_444_736_000_000_000;
const ERROR_NOT_FOUND_HRESULT: i32 = 0x8007_0490_u32 as i32;

pub fn replace_scheduled_toasts(occurrences: &[ReminderOccurrence]) -> Result<usize, AppError> {
    let notifier = notification_notifier()?;
    let existing = notifier
        .GetScheduledToastNotifications()
        .map_err(|error| platform_error("read scheduled notifications", error))?;
    let size = existing
        .Size()
        .map_err(|error| platform_error("read scheduled notification count", error))?;
    let mut reminders = Vec::new();
    for index in 0..size {
        let scheduled = match existing.GetAt(index) {
            Ok(scheduled) => scheduled,
            Err(error) if is_element_not_found(&error) => continue,
            Err(error) => {
                return Err(platform_error(
                    "read scheduled notification snapshot",
                    error,
                ));
            }
        };
        if scheduled
            .Group()
            .map(|group| group == REMINDER_GROUP)
            .unwrap_or(false)
        {
            reminders.push(scheduled);
        }
    }
    for scheduled in reminders {
        if let Err(error) = notifier.RemoveFromSchedule(&scheduled)
            && !is_element_not_found(&error)
        {
            return Err(platform_error("remove scheduled notification", error));
        }
    }

    for occurrence in occurrences {
        let xml = XmlDocument::new()
            .map_err(|error| platform_error("create scheduled notification XML", error))?;
        xml.LoadXml(&HSTRING::from(scheduled_toast_xml(occurrence)))
            .map_err(|error| platform_error("load scheduled notification XML", error))?;
        let delivery = DateTime {
            UniversalTime: occurrence
                .delivery_at_millis
                .saturating_mul(10_000)
                .saturating_add(WINDOWS_EPOCH_TICKS),
        };
        let toast = ScheduledToastNotification::CreateScheduledToastNotification(&xml, delivery)
            .map_err(|error| platform_error("create scheduled notification", error))?;
        toast
            .SetTag(&HSTRING::from(schedule_tag(occurrence)))
            .map_err(|error| platform_error("set scheduled notification tag", error))?;
        toast
            .SetGroup(&HSTRING::from(REMINDER_GROUP))
            .map_err(|error| platform_error("set scheduled notification group", error))?;
        notifier
            .AddToSchedule(&toast)
            .map_err(|error| platform_error("add scheduled notification", error))?;
    }
    Ok(occurrences.len())
}

fn notification_notifier() -> Result<ToastNotifier, AppError> {
    let notifier =
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(APP_USER_MODEL_ID))
            .map_err(|error| platform_error("open PixelDone notification identity", error))?;
    match notifier.Setting() {
        Ok(setting) if setting != NotificationSetting::Enabled => {
            return Err(AppError::NotificationsDisabled(notification_setting_name(
                setting,
            )));
        }
        Ok(_) => {}
        // Some unpackaged desktop identities do not expose a Settings entry
        // until Windows has accepted their first toast. Queue operations remain
        // authoritative and surface a real failure if the identity is invalid.
        Err(error) if is_element_not_found(&error) => {}
        Err(error) => {
            return Err(platform_error("read Windows notification setting", error));
        }
    }
    Ok(notifier)
}

fn notification_setting_name(setting: NotificationSetting) -> String {
    if setting == NotificationSetting::DisabledForApplication {
        "Disabled for PixelDone in Windows Settings".to_owned()
    } else if setting == NotificationSetting::DisabledForUser {
        "Disabled for the current Windows user".to_owned()
    } else if setting == NotificationSetting::DisabledByGroupPolicy {
        "Disabled by Windows group policy".to_owned()
    } else if setting == NotificationSetting::DisabledByManifest {
        "Disabled by application registration".to_owned()
    } else {
        format!("Unknown Windows notification setting ({})", setting.0)
    }
}

fn schedule_tag(occurrence: &ReminderOccurrence) -> String {
    let digest = format!(
        "{:x}",
        Sha256::digest(format!(
            "{}:{}",
            occurrence.todo_id, occurrence.delivery_at_millis
        ))
    );
    format!("pd-{}", &digest[..32])
}

fn scheduled_toast_xml(occurrence: &ReminderOccurrence) -> String {
    let encoded_id =
        url::form_urlencoded::byte_serialize(occurrence.todo_id.as_bytes()).collect::<String>();
    let open = format!("pixeldone-reminder://open?todo={encoded_id}");
    let stop = format!("pixeldone-reminder://stop?todo={encoded_id}");
    let snooze = format!("pixeldone-reminder://snooze?todo={encoded_id}");
    let (scenario, audio) = if occurrence.enhanced_alarm {
        (
            " scenario=\"alarm\"",
            "<audio src=\"ms-winsoundevent:Notification.Looping.Alarm2\" loop=\"true\"/>",
        )
    } else {
        ("", "<audio src=\"ms-winsoundevent:Notification.Default\"/>")
    };
    format!(
        "<toast{scenario} activationType=\"protocol\" launch=\"{}\"><visual><binding template=\"ToastGeneric\"><text>PixelDone</text><text>{}</text></binding></visual><actions><action content=\"STOP\" arguments=\"{}\" activationType=\"protocol\"/><action content=\"SNOOZE 10 MIN\" arguments=\"{}\" activationType=\"protocol\"/></actions>{audio}</toast>",
        escape_xml(&open),
        escape_xml(&occurrence.title),
        escape_xml(&stop),
        escape_xml(&snooze),
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn is_element_not_found(error: &windows::core::Error) -> bool {
    error.code().0 == ERROR_NOT_FOUND_HRESULT
}

pub fn is_element_not_found_app_error(error: &AppError) -> bool {
    matches!(error, AppError::Platform(message) if message.contains("0x80070490") || message.contains("Element not found"))
}

fn platform_error(context: &str, error: windows::core::Error) -> AppError {
    AppError::Platform(format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_toast_xml_escapes_user_titles() {
        assert_eq!(escape_xml("A & <B>"), "A &amp; &lt;B&gt;");
        let xml = scheduled_toast_xml(&ReminderOccurrence {
            todo_id: "task".into(),
            title: "A & <B>".into(),
            priority: crate::domain::TodoPriority::Medium,
            delivery_at_millis: 1,
            enhanced_alarm: false,
        });
        assert!(xml.contains("A &amp; &lt;B&gt;"));
        assert!(xml.contains("pixeldone-reminder://stop"));
        assert!(xml.contains("pixeldone-reminder://snooze"));
    }

    #[test]
    fn xhigh_uses_standard_notification_when_enhancement_is_disabled() {
        let xml = scheduled_toast_xml(&ReminderOccurrence {
            todo_id: "a&b".into(),
            title: "Wake <now>".into(),
            priority: crate::domain::TodoPriority::Xhigh,
            delivery_at_millis: 1,
            enhanced_alarm: false,
        });
        assert!(!xml.contains("scenario=\"alarm\""));
        assert!(!xml.contains("Notification.Looping.Alarm2"));
        assert!(xml.contains("Notification.Default"));
        assert!(xml.contains("activationType=\"protocol\""));
    }

    #[test]
    fn enhanced_xhigh_uses_alarm_scenario_and_protocol_actions() {
        let xml = scheduled_toast_xml(&ReminderOccurrence {
            todo_id: "a&b".into(),
            title: "Wake <now>".into(),
            priority: crate::domain::TodoPriority::Xhigh,
            delivery_at_millis: 1,
            enhanced_alarm: true,
        });
        assert!(xml.contains("scenario=\"alarm\""));
        assert!(xml.contains("Notification.Looping.Alarm2"));
        assert!(xml.contains("activationType=\"protocol\""));
        assert!(xml.contains("a%26b"));
    }

    #[test]
    fn element_not_found_is_recognized_for_idempotent_queue_operations() {
        let error = AppError::Platform(
            "remove scheduled notification: Element not found. (0x80070490)".to_owned(),
        );
        assert!(is_element_not_found_app_error(&error));
        assert!(!is_element_not_found_app_error(&AppError::Platform(
            "Access denied. (0x80070005)".to_owned()
        )));
    }
}
