use crate::domain::{ReminderRepeat, TodoItem};

pub const DAILY_INTERVAL_MILLIS: i64 = 24 * 60 * 60 * 1000;
pub const WEEKLY_INTERVAL_MILLIS: i64 = 7 * DAILY_INTERVAL_MILLIS;
pub const DEFAULT_SNOOZE_MILLIS: i64 = 10 * 60 * 1000;

pub fn next_reminder_at(item: &TodoItem, now_millis: i64) -> Option<i64> {
    if item.completed || item.is_trashed() || item.due_at_millis <= 0 {
        return None;
    }
    match item.reminder_repeat {
        ReminderRepeat::None => (item.due_at_millis > now_millis).then_some(item.due_at_millis),
        ReminderRepeat::Daily => {
            next_repeating(item.due_at_millis, DAILY_INTERVAL_MILLIS, now_millis)
        }
        ReminderRepeat::Weekly => {
            next_repeating(item.due_at_millis, WEEKLY_INTERVAL_MILLIS, now_millis)
        }
    }
}

fn next_repeating(due: i64, interval: i64, now: i64) -> Option<i64> {
    if due > now {
        return Some(due);
    }
    let elapsed_intervals = (now - due) / interval;
    let next = due + (elapsed_intervals + 1) * interval;
    (next > now).then_some(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ReminderRepeat, TodoPriority};

    #[test]
    fn daily_reminder_advances_to_first_future_interval() {
        let item = TodoItem {
            id: "1".into(),
            title: "daily".into(),
            priority: TodoPriority::Medium,
            due_at_millis: 100,
            completed: false,
            created_at_millis: 1,
            reminder_repeat: ReminderRepeat::Daily,
            image_file_name: None,
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
        };
        assert_eq!(
            next_reminder_at(&item, 101),
            Some(100 + DAILY_INTERVAL_MILLIS)
        );
    }
}
