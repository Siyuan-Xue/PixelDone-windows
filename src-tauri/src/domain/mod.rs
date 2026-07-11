pub mod checklist;
pub mod dock;
pub mod error;
pub mod reminder;
pub mod sync;
pub mod todo;

use serde::{Deserialize, Serialize};

pub use checklist::{Checklist, ChecklistKind};
pub use dock::{DockAction, DockConfig, DockPlusPlacement};
pub use error::AppError;
pub use sync::{
    AppLanguage, AuthView, ConflictResolutionChoice, ReminderRunView, SyncConflictView,
    SyncRunView, SyncState, UpdateView,
};
pub use todo::{ReminderRepeat, SortMode, TodoItem, TodoPriority};

pub const DEFAULT_CHECKLIST_ID: &str = "main";
pub const DEFAULT_CHECKLIST_NAME: &str = "MAIN";
pub const TRASH_CHECKLIST_ID: &str = "trash";
pub const TRASH_CHECKLIST_NAME: &str = "TRASH";
pub const SETTINGS_CHECKLIST_ID: &str = "settings";
pub const SETTINGS_CHECKLIST_NAME: &str = "SETTINGS";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub dark_theme: bool,
    pub dock: DockConfig,
    pub never_show_update_dialog: bool,
    pub future_sync_enabled: bool,
    pub language_mode: AppLanguage,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub revision: i64,
    pub checklists: Vec<Checklist>,
    pub selected_checklist_id: String,
    pub sort_mode: SortMode,
    pub hide_completed: bool,
    pub quick_delete: bool,
    pub show_deadline_countdown: bool,
    pub checklist_history: Vec<String>,
    pub settings: AppSettings,
    pub auth: AuthView,
    pub sync: SyncRunView,
    pub reminder: ReminderRunView,
    pub update: UpdateView,
}

impl AppSnapshot {
    pub fn initial(now_millis: i64) -> Self {
        Self {
            revision: 0,
            checklists: vec![
                Checklist::new_normal(
                    DEFAULT_CHECKLIST_ID.to_owned(),
                    DEFAULT_CHECKLIST_NAME.to_owned(),
                    now_millis,
                ),
                Checklist::new_special(
                    TRASH_CHECKLIST_ID,
                    TRASH_CHECKLIST_NAME,
                    ChecklistKind::Trash,
                    now_millis,
                ),
                Checklist::new_special(
                    SETTINGS_CHECKLIST_ID,
                    SETTINGS_CHECKLIST_NAME,
                    ChecklistKind::Settings,
                    now_millis,
                ),
            ],
            selected_checklist_id: DEFAULT_CHECKLIST_ID.to_owned(),
            sort_mode: SortMode::Priority,
            hide_completed: false,
            quick_delete: false,
            show_deadline_countdown: false,
            checklist_history: Vec::new(),
            settings: AppSettings::default(),
            auth: AuthView {
                cloud_available: true,
                insecure_http: true,
                ..AuthView::default()
            },
            sync: SyncRunView {
                state: SyncState::SignedOut,
                insecure_http: true,
                ..SyncRunView::default()
            },
            reminder: ReminderRunView {
                state: "IDLE".to_owned(),
                ..ReminderRunView::default()
            },
            update: UpdateView {
                state: "IDLE".to_owned(),
                current_version: env!("CARGO_PKG_VERSION").to_owned(),
                ..UpdateView::default()
            },
        }
    }

    pub fn selected_checklist(&self) -> Option<&Checklist> {
        self.checklists
            .iter()
            .find(|list| list.id == self.selected_checklist_id)
    }

    pub fn checklist_mut(&mut self, id: &str) -> Result<&mut Checklist, AppError> {
        self.checklists
            .iter_mut()
            .find(|list| list.id == id)
            .ok_or_else(|| AppError::NotFound(format!("checklist {id}")))
    }

    pub fn normal_checklist_count(&self) -> usize {
        self.checklists
            .iter()
            .filter(|list| list.kind == ChecklistKind::Normal)
            .count()
    }

    pub fn normalized(mut self) -> Self {
        let now = unix_now_millis();
        self.checklists.retain(|list| !list.id.trim().is_empty());
        self.checklists
            .retain(|list| !list.name.trim().is_empty() || list.kind != ChecklistKind::Normal);

        if !self
            .checklists
            .iter()
            .any(|list| list.kind == ChecklistKind::Normal)
        {
            self.checklists.push(Checklist::new_normal(
                DEFAULT_CHECKLIST_ID.to_owned(),
                DEFAULT_CHECKLIST_NAME.to_owned(),
                now,
            ));
        }
        ensure_special(&mut self.checklists, ChecklistKind::Trash, now);
        ensure_special(&mut self.checklists, ChecklistKind::Settings, now);
        self.checklists.sort_by_key(|list| match list.kind {
            ChecklistKind::Normal => 0,
            ChecklistKind::Trash => 1,
            ChecklistKind::Settings => 2,
        });
        if !self
            .checklists
            .iter()
            .any(|list| list.id == self.selected_checklist_id)
        {
            self.selected_checklist_id = self
                .checklists
                .iter()
                .find(|list| list.kind == ChecklistKind::Normal)
                .map(|list| list.id.clone())
                .unwrap_or_else(|| DEFAULT_CHECKLIST_ID.to_owned());
        }
        self
    }
}

fn ensure_special(checklists: &mut Vec<Checklist>, kind: ChecklistKind, now: i64) {
    if checklists.iter().any(|list| list.kind == kind) {
        return;
    }
    let (id, name) = match kind {
        ChecklistKind::Trash => (TRASH_CHECKLIST_ID, TRASH_CHECKLIST_NAME),
        ChecklistKind::Settings => (SETTINGS_CHECKLIST_ID, SETTINGS_CHECKLIST_NAME),
        ChecklistKind::Normal => return,
    };
    checklists.push(Checklist::new_special(id, name, kind, now));
}

pub fn unix_now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDelta {
    pub upserted_checklists: Vec<Checklist>,
    pub removed_checklist_ids: Vec<String>,
    pub selected_checklist_id: Option<String>,
    pub sort_mode: Option<SortMode>,
    pub hide_completed: Option<bool>,
    pub quick_delete: Option<bool>,
    pub show_deadline_countdown: Option<bool>,
    pub checklist_history: Option<Vec<String>>,
    pub settings: Option<AppSettings>,
    pub auth: Option<AuthView>,
    pub sync: Option<SyncRunView>,
    pub reminder: Option<ReminderRunView>,
    pub update: Option<UpdateView>,
}

impl SnapshotDelta {
    pub fn between(before: &AppSnapshot, after: &AppSnapshot) -> Self {
        let upserted_checklists = after
            .checklists
            .iter()
            .filter(|list| before.checklists.iter().find(|old| old.id == list.id) != Some(*list))
            .cloned()
            .collect();
        let removed_checklist_ids = before
            .checklists
            .iter()
            .filter(|list| !after.checklists.iter().any(|new| new.id == list.id))
            .map(|list| list.id.clone())
            .collect();

        Self {
            upserted_checklists,
            removed_checklist_ids,
            selected_checklist_id: (before.selected_checklist_id != after.selected_checklist_id)
                .then(|| after.selected_checklist_id.clone()),
            sort_mode: (before.sort_mode != after.sort_mode).then_some(after.sort_mode),
            hide_completed: (before.hide_completed != after.hide_completed)
                .then_some(after.hide_completed),
            quick_delete: (before.quick_delete != after.quick_delete).then_some(after.quick_delete),
            show_deadline_countdown: (before.show_deadline_countdown
                != after.show_deadline_countdown)
                .then_some(after.show_deadline_countdown),
            checklist_history: (before.checklist_history != after.checklist_history)
                .then(|| after.checklist_history.clone()),
            settings: (before.settings != after.settings).then(|| after.settings.clone()),
            auth: (before.auth != after.auth).then(|| after.auth.clone()),
            sync: (before.sync != after.sync).then(|| after.sync.clone()),
            reminder: (before.reminder != after.reminder).then(|| after.reminder.clone()),
            update: (before.update != after.update).then(|| after.update.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub revision: i64,
    pub changed_ids: Vec<String>,
    pub snapshot_delta: SnapshotDelta,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state_always_has_one_normal_and_two_special_lists() {
        let snapshot = AppSnapshot::initial(42);
        assert_eq!(snapshot.normal_checklist_count(), 1);
        assert!(
            snapshot
                .checklists
                .iter()
                .any(|list| list.kind == ChecklistKind::Trash)
        );
        assert!(
            snapshot
                .checklists
                .iter()
                .any(|list| list.kind == ChecklistKind::Settings)
        );
    }
}
