use serde::{Deserialize, Serialize};

use crate::domain::todo::{SortMode, TodoItem, sort_todos};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChecklistKind {
    Normal,
    Trash,
    Settings,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Checklist {
    pub id: String,
    pub name: String,
    pub kind: ChecklistKind,
    pub items: Vec<TodoItem>,
    pub created_at_millis: i64,
    pub updated_at_millis: i64,
    pub remote_version: Option<i64>,
}

impl Checklist {
    pub fn new_normal(id: String, name: String, created_at_millis: i64) -> Self {
        Self {
            id,
            name,
            kind: ChecklistKind::Normal,
            items: Vec::new(),
            created_at_millis,
            updated_at_millis: created_at_millis,
            remote_version: None,
        }
    }

    pub fn new_special(id: &str, name: &str, kind: ChecklistKind, created_at_millis: i64) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            kind,
            items: Vec::new(),
            created_at_millis,
            updated_at_millis: created_at_millis,
            remote_version: None,
        }
    }

    pub fn active_count(&self) -> usize {
        self.items.iter().filter(|item| !item.completed).count()
    }

    pub fn sorted_items(&self, mode: SortMode, hide_completed: bool) -> Vec<TodoItem> {
        let mut items = self
            .items
            .iter()
            .filter(|item| !hide_completed || !item.completed)
            .cloned()
            .collect::<Vec<_>>();
        sort_todos(&mut items, mode);
        items
    }
}
