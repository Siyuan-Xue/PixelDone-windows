use serde::{Deserialize, Serialize};

use crate::domain::AppError;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TodoPriority {
    Xhigh,
    High,
    #[default]
    Medium,
    Low,
}

impl TodoPriority {
    pub fn rank(self) -> u8 {
        match self {
            Self::Xhigh => 0,
            Self::High => 1,
            Self::Medium => 2,
            Self::Low => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReminderRepeat {
    #[default]
    None,
    Daily,
    Weekly,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SortMode {
    #[default]
    Priority,
    Time,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoItem {
    pub id: String,
    pub title: String,
    pub priority: TodoPriority,
    pub due_at_millis: i64,
    pub completed: bool,
    pub created_at_millis: i64,
    pub reminder_repeat: ReminderRepeat,
    pub image_file_name: Option<String>,
    pub trashed_from_checklist_id: Option<String>,
    pub trashed_from_checklist_name: Option<String>,
    pub trashed_at_millis: Option<i64>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoDraft {
    pub title: String,
    pub priority: TodoPriority,
    pub due_at_millis: i64,
    pub reminder_repeat: ReminderRepeat,
}

impl TodoDraft {
    pub fn validate(self) -> Result<Self, AppError> {
        if self.title.trim().is_empty() {
            return Err(AppError::Validation("任务标题不能为空".to_owned()));
        }
        Ok(Self {
            title: self.title.trim().to_owned(),
            ..self
        })
    }
}

impl TodoItem {
    pub fn from_draft(
        id: String,
        draft: TodoDraft,
        created_at_millis: i64,
    ) -> Result<Self, AppError> {
        let draft = draft.validate()?;
        Ok(Self {
            id,
            title: draft.title,
            priority: draft.priority,
            due_at_millis: draft.due_at_millis,
            completed: false,
            created_at_millis,
            reminder_repeat: draft.reminder_repeat,
            image_file_name: None,
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
        })
    }

    pub fn is_trashed(&self) -> bool {
        self.trashed_at_millis.is_some() || self.trashed_from_checklist_id.is_some()
    }

    pub fn apply_draft(&mut self, draft: TodoDraft) -> Result<(), AppError> {
        let draft = draft.validate()?;
        self.title = draft.title;
        self.priority = draft.priority;
        self.due_at_millis = draft.due_at_millis;
        self.reminder_repeat = draft.reminder_repeat;
        Ok(())
    }
}

pub fn sort_todos(items: &mut [TodoItem], mode: SortMode) {
    items.sort_by_key(|item| match mode {
        SortMode::Priority => (
            item.completed,
            item.priority.rank(),
            item.due_at_millis,
            item.created_at_millis,
        ),
        SortMode::Time => (
            item.completed,
            0,
            item.due_at_millis,
            (item.priority.rank() as i64) * 10_000_000_000_000 + item.created_at_millis,
        ),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, priority: TodoPriority, due: i64, completed: bool) -> TodoItem {
        TodoItem {
            id: id.to_owned(),
            title: id.to_owned(),
            priority,
            due_at_millis: due,
            completed,
            created_at_millis: due,
            reminder_repeat: ReminderRepeat::None,
            image_file_name: None,
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
        }
    }

    #[test]
    fn priority_sort_matches_android_completed_priority_due_order() {
        let mut items = vec![
            item("done", TodoPriority::Xhigh, 1, true),
            item("low", TodoPriority::Low, 1, false),
            item("xhigh", TodoPriority::Xhigh, 2, false),
        ];
        sort_todos(&mut items, SortMode::Priority);
        assert_eq!(
            items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["xhigh", "low", "done"]
        );
    }

    #[test]
    fn blank_title_is_rejected_after_trim() {
        let draft = TodoDraft {
            title: "   ".to_owned(),
            priority: TodoPriority::Low,
            due_at_millis: 0,
            reminder_repeat: ReminderRepeat::None,
        };
        assert!(draft.validate().is_err());
    }
}
