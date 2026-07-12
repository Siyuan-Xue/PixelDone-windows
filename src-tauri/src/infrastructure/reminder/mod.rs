//! Platform-neutral reminder occurrence generation.

use crate::domain::{AppSnapshot, ChecklistKind, ReminderRepeat, TodoPriority};

pub const SCHEDULE_LIMIT: usize = 4_000;
pub const SCHEDULE_HORIZON_MILLIS: i64 = 366 * 24 * 60 * 60 * 1_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderOccurrence {
    pub todo_id: String,
    pub title: String,
    pub priority: TodoPriority,
    pub delivery_at_millis: i64,
    pub enhanced_alarm: bool,
}

pub fn occurrences(snapshot: &AppSnapshot, now_millis: i64) -> (Vec<ReminderOccurrence>, bool) {
    let horizon = now_millis + SCHEDULE_HORIZON_MILLIS;
    let mut values = snapshot
        .checklists
        .iter()
        .filter(|list| list.kind == ChecklistKind::Normal)
        .flat_map(|list| list.items.iter())
        .filter(|item| !item.completed && !item.is_trashed())
        .flat_map(|item| {
            let interval = match item.reminder_repeat {
                ReminderRepeat::None => None,
                ReminderRepeat::Daily => Some(crate::domain::reminder::DAILY_INTERVAL_MILLIS),
                ReminderRepeat::Weekly => Some(crate::domain::reminder::WEEKLY_INTERVAL_MILLIS),
            };
            let mut times = Vec::new();
            let Some(mut at) = crate::domain::reminder::next_reminder_at(item, now_millis) else {
                return Vec::new();
            };
            while at <= horizon {
                times.push(ReminderOccurrence {
                    todo_id: item.id.clone(),
                    title: item.title.clone(),
                    priority: item.priority,
                    delivery_at_millis: at,
                    enhanced_alarm: snapshot.settings.enhanced_xhigh_alarm_enabled
                        && item.priority == TodoPriority::Xhigh,
                });
                let Some(interval) = interval else { break };
                at += interval;
            }
            times
        })
        .collect::<Vec<_>>();
    values.sort_by_key(|value| value.delivery_at_millis);
    let truncated = values.len() > SCHEDULE_LIMIT;
    values.truncate(SCHEDULE_LIMIT);
    (values, truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{AppSnapshot, ReminderRepeat, TodoItem};

    #[test]
    fn repeating_occurrences_are_sorted_and_bounded() {
        let mut snapshot = AppSnapshot::initial(1);
        snapshot.checklists[0].items.push(TodoItem {
            id: "daily".into(),
            title: "Daily".into(),
            priority: TodoPriority::High,
            due_at_millis: 1_000,
            completed: false,
            created_at_millis: 1,
            updated_at_millis: 1,
            reminder_repeat: ReminderRepeat::Daily,
            image_file_name: None,
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
            remote_version: None,
        });
        let (values, truncated) = occurrences(&snapshot, 100);
        assert!(!truncated);
        assert!(values.len() > 300);
        assert_eq!(values[0].delivery_at_millis, 1_000);
    }

    #[test]
    fn xhigh_notification_policy_is_device_local_and_defaults_to_standard() {
        let mut snapshot = AppSnapshot::initial(1);
        snapshot.checklists[0].items.push(TodoItem {
            id: "xhigh".into(),
            title: "Important".into(),
            priority: TodoPriority::Xhigh,
            due_at_millis: 10_000,
            completed: false,
            created_at_millis: 1,
            updated_at_millis: 1,
            reminder_repeat: ReminderRepeat::None,
            image_file_name: None,
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
            remote_version: None,
        });
        let (standard, _) = occurrences(&snapshot, 100);
        assert_eq!(standard.len(), 1);
        assert!(!standard[0].enhanced_alarm);

        snapshot.settings.enhanced_xhigh_alarm_enabled = true;
        let (enhanced, _) = occurrences(&snapshot, 100);
        assert!(enhanced[0].enhanced_alarm);

        snapshot.checklists[0].items[0].completed = true;
        assert!(occurrences(&snapshot, 100).0.is_empty());
    }

    #[test]
    fn complete_snapshot_rebuild_replaces_and_revokes_remote_reminders() {
        let mut snapshot = AppSnapshot::initial(1);
        snapshot.checklists[0].items.push(TodoItem {
            id: "remote".into(),
            title: "Remote task".into(),
            priority: TodoPriority::Medium,
            due_at_millis: 10_000,
            completed: false,
            created_at_millis: 1,
            updated_at_millis: 1,
            reminder_repeat: ReminderRepeat::None,
            image_file_name: None,
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
            remote_version: Some(1),
        });

        let (created, _) = occurrences(&snapshot, 100);
        assert_eq!(created[0].delivery_at_millis, 10_000);

        snapshot.checklists[0].items[0].due_at_millis = 20_000;
        snapshot.checklists[0].items[0].remote_version = Some(2);
        let (rescheduled, _) = occurrences(&snapshot, 100);
        assert_eq!(rescheduled.len(), 1);
        assert_eq!(rescheduled[0].delivery_at_millis, 20_000);

        snapshot.checklists[0].items.clear();
        assert!(occurrences(&snapshot, 100).0.is_empty());
    }
}
