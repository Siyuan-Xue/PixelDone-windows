use tauri::State;
use tauri_plugin_autostart::ManagerExt;
use uuid::Uuid;

use crate::{
    application::state::ManagedAppState,
    domain::todo::TodoDraft,
    domain::{
        AppError, AppSettings, AppSnapshot, Checklist, ChecklistKind, MutationResult,
        SETTINGS_CHECKLIST_ID, SnapshotDelta, SortMode, TRASH_CHECKLIST_ID, TodoItem,
        unix_now_millis,
    },
};

pub mod auth;
pub mod image;
pub mod reminder;
pub mod storage;
pub mod sync;
pub mod update;

pub(super) async fn ensure_revision(
    state: &State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<(), AppError> {
    require_revision(
        state.inner.lock().await.snapshot.revision,
        expected_revision,
    )
}

fn require_revision(actual_revision: i64, expected_revision: i64) -> Result<(), AppError> {
    if actual_revision == expected_revision {
        Ok(())
    } else {
        Err(AppError::StaleRevision)
    }
}

#[cfg(test)]
mod revision_tests {
    use super::*;

    #[test]
    fn stale_preflight_stops_work_before_a_side_effect() {
        let mut side_effect_started = false;
        if require_revision(9, 8).is_ok() {
            side_effect_started = true;
        }
        assert!(!side_effect_started);
        assert!(require_revision(9, 9).is_ok());
    }

    #[test]
    fn restoring_from_a_deleted_checklist_creates_a_new_sync_identity() {
        let mut snapshot = AppSnapshot::initial(1);
        let mut first = TodoItem::from_draft(
            "first".to_owned(),
            TodoDraft {
                title: "First".to_owned(),
                priority: crate::domain::TodoPriority::Low,
                due_at_millis: 0,
                reminder_repeat: crate::domain::ReminderRepeat::None,
            },
            2,
        )
        .unwrap();
        first.trashed_from_checklist_id = Some("deleted-list".to_owned());
        first.trashed_from_checklist_name = Some("Deleted list".to_owned());
        first.trashed_at_millis = Some(3);
        let mut second = first.clone();
        second.id = "second".to_owned();
        snapshot.checklist_mut(TRASH_CHECKLIST_ID).unwrap().items = vec![first, second];

        let changed = restore_todo_in_snapshot(&mut snapshot, "first", 4).unwrap();
        let restored = snapshot
            .checklists
            .iter()
            .find(|list| list.kind == ChecklistKind::Normal && list.name == "Deleted list")
            .unwrap();
        let restored_id = restored.id.clone();

        assert_ne!(restored_id, "deleted-list");
        assert_eq!(restored.items[0].id, "first");
        assert!(changed.contains(&restored_id));
        assert_eq!(
            snapshot.checklist_mut(TRASH_CHECKLIST_ID).unwrap().items[0]
                .trashed_from_checklist_id
                .as_deref(),
            Some(restored_id.as_str())
        );
    }
}

pub(super) async fn mutate<F>(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    operation: F,
) -> Result<MutationResult, AppError>
where
    F: FnOnce(&mut AppSnapshot) -> Result<Vec<String>, AppError>,
{
    let mut runtime = state.inner.lock().await;
    if runtime.snapshot.revision != expected_revision {
        return Err(AppError::StaleRevision);
    }

    let before = runtime.snapshot.clone();
    let mut candidate = before.clone();
    let changed_ids = operation(&mut candidate)?;
    candidate.revision += 1;
    runtime
        .repository
        .save_snapshot_with_changes(&before, &candidate)
        .await?;
    let snapshot_delta = SnapshotDelta::between(&before, &candidate);
    runtime.snapshot = candidate;
    state.sync_notify.notify_one();
    state.reminder_notify.notify_one();

    Ok(MutationResult {
        revision: runtime.snapshot.revision,
        changed_ids,
        snapshot_delta,
    })
}

#[tauri::command]
pub async fn bootstrap(state: State<'_, ManagedAppState>) -> Result<AppSnapshot, AppError> {
    Ok(state.inner.lock().await.snapshot.clone())
}

#[tauri::command]
pub async fn select_checklist(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        if !snapshot
            .checklists
            .iter()
            .any(|list| list.id == checklist_id)
        {
            return Err(AppError::NotFound(format!("checklist {checklist_id}")));
        }
        let previous = snapshot.selected_checklist_id.clone();
        snapshot.selected_checklist_id = checklist_id.clone();
        if previous != checklist_id {
            snapshot.checklist_history.push(previous);
            if snapshot.checklist_history.len() > 50 {
                snapshot.checklist_history.remove(0);
            }
        }
        Ok(vec![checklist_id])
    })
    .await
}

#[tauri::command]
pub async fn back_checklist(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        while let Some(checklist_id) = snapshot.checklist_history.pop() {
            if snapshot
                .checklists
                .iter()
                .any(|list| list.id == checklist_id)
            {
                snapshot.selected_checklist_id = checklist_id.clone();
                return Ok(vec![checklist_id]);
            }
        }
        Err(AppError::Validation("没有可返回的清单".to_owned()))
    })
    .await
}

#[tauri::command]
pub async fn create_checklist(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    name: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let name = validated_checklist_name(snapshot, &name, None)?;
        let id = Uuid::new_v4().to_string();
        let insert_at = snapshot
            .checklists
            .iter()
            .position(|list| list.kind != ChecklistKind::Normal)
            .unwrap_or(snapshot.checklists.len());
        snapshot.checklists.insert(
            insert_at,
            Checklist::new_normal(id.clone(), name, unix_now_millis()),
        );
        snapshot.selected_checklist_id = id.clone();
        Ok(vec![id])
    })
    .await
}

#[tauri::command]
pub async fn rename_checklist(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    name: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let name = validated_checklist_name(snapshot, &name, Some(&checklist_id))?;
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        if checklist.kind != ChecklistKind::Normal {
            return Err(AppError::Validation("特殊清单不能重命名".to_owned()));
        }
        checklist.name = name;
        checklist.updated_at_millis = unix_now_millis();
        Ok(vec![checklist_id])
    })
    .await
}

#[tauri::command]
pub async fn delete_checklist(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        if snapshot.normal_checklist_count() <= 1 {
            return Err(AppError::Validation("至少保留一个普通清单".to_owned()));
        }
        let source_index = snapshot
            .checklists
            .iter()
            .position(|list| list.id == checklist_id && list.kind == ChecklistKind::Normal)
            .ok_or_else(|| AppError::NotFound(format!("checklist {checklist_id}")))?;
        let source = snapshot.checklists.remove(source_index);
        let now = unix_now_millis();
        let source_name = source.name.clone();
        let trash = snapshot.checklist_mut(TRASH_CHECKLIST_ID)?;
        for mut item in source.items {
            item.trashed_from_checklist_id = Some(source.id.clone());
            item.trashed_from_checklist_name = Some(source_name.clone());
            item.trashed_at_millis = Some(now);
            item.updated_at_millis = now;
            trash.items.push(item);
        }
        if snapshot.selected_checklist_id == checklist_id {
            snapshot.selected_checklist_id = snapshot
                .checklists
                .iter()
                .find(|list| list.kind == ChecklistKind::Normal)
                .expect("normal checklist remains")
                .id
                .clone();
        }
        Ok(vec![checklist_id, TRASH_CHECKLIST_ID.to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn create_todo(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    draft: TodoDraft,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let id = Uuid::new_v4().to_string();
        let item = TodoItem::from_draft(id.clone(), draft, unix_now_millis())?;
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        if checklist.kind != ChecklistKind::Normal {
            return Err(AppError::Validation("只能在普通清单中创建任务".to_owned()));
        }
        checklist.items.push(item);
        Ok(vec![checklist_id, id])
    })
    .await
}

#[tauri::command]
pub async fn update_todo(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    todo_id: String,
    draft: TodoDraft,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        let item = checklist
            .items
            .iter_mut()
            .find(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        item.apply_draft(draft)?;
        Ok(vec![checklist_id, todo_id])
    })
    .await
}

#[tauri::command]
pub async fn toggle_todo(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    todo_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        let item = checklist
            .items
            .iter_mut()
            .find(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        item.completed = !item.completed;
        item.updated_at_millis = unix_now_millis();
        Ok(vec![checklist_id, todo_id])
    })
    .await
}

#[tauri::command]
pub async fn move_todo_to_trash(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    todo_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let source_index = snapshot
            .checklists
            .iter()
            .position(|list| list.id == checklist_id && list.kind == ChecklistKind::Normal)
            .ok_or_else(|| AppError::NotFound(format!("checklist {checklist_id}")))?;
        let todo_index = snapshot.checklists[source_index]
            .items
            .iter()
            .position(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        let source_id = snapshot.checklists[source_index].id.clone();
        let source_name = snapshot.checklists[source_index].name.clone();
        let mut item = snapshot.checklists[source_index].items.remove(todo_index);
        item.trashed_from_checklist_id = Some(source_id);
        item.trashed_from_checklist_name = Some(source_name);
        let now = unix_now_millis();
        item.trashed_at_millis = Some(now);
        item.updated_at_millis = now;
        snapshot.checklist_mut(TRASH_CHECKLIST_ID)?.items.push(item);
        Ok(vec![checklist_id, TRASH_CHECKLIST_ID.to_owned(), todo_id])
    })
    .await
}

#[tauri::command]
pub async fn clean_completed(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let source_index = snapshot
            .checklists
            .iter()
            .position(|list| list.id == checklist_id && list.kind == ChecklistKind::Normal)
            .ok_or_else(|| AppError::NotFound(format!("checklist {checklist_id}")))?;
        let source_id = snapshot.checklists[source_index].id.clone();
        let source_name = snapshot.checklists[source_index].name.clone();
        let mut retained = Vec::new();
        let mut moved = Vec::new();
        for mut item in snapshot.checklists[source_index].items.drain(..) {
            if item.completed {
                item.trashed_from_checklist_id = Some(source_id.clone());
                item.trashed_from_checklist_name = Some(source_name.clone());
                let now = unix_now_millis();
                item.trashed_at_millis = Some(now);
                item.updated_at_millis = now;
                moved.push(item);
            } else {
                retained.push(item);
            }
        }
        if moved.is_empty() {
            return Err(AppError::Validation("没有已完成任务可清理".to_owned()));
        }
        snapshot.checklists[source_index].items = retained;
        let moved_ids = moved.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
        snapshot
            .checklist_mut(TRASH_CHECKLIST_ID)?
            .items
            .extend(moved);
        let mut changed = vec![checklist_id, TRASH_CHECKLIST_ID.to_owned()];
        changed.extend(moved_ids);
        Ok(changed)
    })
    .await
}

#[tauri::command]
pub async fn restore_todo(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    todo_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        restore_todo_in_snapshot(snapshot, &todo_id, unix_now_millis())
    })
    .await
}

fn restore_todo_in_snapshot(
    snapshot: &mut AppSnapshot,
    todo_id: &str,
    restored_at_millis: i64,
) -> Result<Vec<String>, AppError> {
    let trash_index = snapshot
        .checklists
        .iter()
        .position(|list| list.id == TRASH_CHECKLIST_ID)
        .ok_or_else(|| AppError::NotFound("trash checklist".to_owned()))?;
    let todo_index = snapshot.checklists[trash_index]
        .items
        .iter()
        .position(|item| item.id == todo_id)
        .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
    let mut item = snapshot.checklists[trash_index].items.remove(todo_index);
    let original_id = item
        .trashed_from_checklist_id
        .clone()
        .filter(|id| id != TRASH_CHECKLIST_ID && id != SETTINGS_CHECKLIST_ID);
    let original_exists = original_id.as_ref().is_some_and(|id| {
        snapshot
            .checklists
            .iter()
            .any(|list| list.id == *id && list.kind == ChecklistKind::Normal)
    });
    let target_id = if original_exists {
        original_id
            .clone()
            .expect("existing original checklist has an id")
    } else if original_id.is_some() {
        // Cloud tombstones are permanent. A restore is therefore a new record with the
        // original display name, never an update that competes with the deleted identity.
        Uuid::new_v4().to_string()
    } else {
        snapshot
            .checklists
            .iter()
            .find(|list| list.kind == ChecklistKind::Normal)
            .map(|list| list.id.clone())
            .unwrap_or_else(|| "main".to_owned())
    };

    let mut changed = vec![
        TRASH_CHECKLIST_ID.to_owned(),
        target_id.clone(),
        todo_id.to_owned(),
    ];
    if !snapshot.checklists.iter().any(|list| list.id == target_id) {
        let name = item
            .trashed_from_checklist_name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "MAIN".to_owned());
        let insert_at = snapshot
            .checklists
            .iter()
            .position(|list| list.kind != ChecklistKind::Normal)
            .unwrap_or(snapshot.checklists.len());
        snapshot.checklists.insert(
            insert_at,
            Checklist::new_normal(target_id.clone(), name, restored_at_millis),
        );

        if let Some(original_id) = original_id.as_deref() {
            let trash = snapshot.checklist_mut(TRASH_CHECKLIST_ID)?;
            for remaining in &mut trash.items {
                if remaining.trashed_from_checklist_id.as_deref() == Some(original_id) {
                    remaining.trashed_from_checklist_id = Some(target_id.clone());
                    remaining.updated_at_millis = restored_at_millis;
                    changed.push(remaining.id.clone());
                }
            }
        }
    }
    item.trashed_from_checklist_id = None;
    item.trashed_from_checklist_name = None;
    item.trashed_at_millis = None;
    item.updated_at_millis = restored_at_millis;
    snapshot.checklist_mut(&target_id)?.items.push(item);
    Ok(changed)
}

#[tauri::command]
pub async fn purge_todo(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    todo_id: String,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let trash = snapshot.checklist_mut(TRASH_CHECKLIST_ID)?;
        let old_len = trash.items.len();
        trash.items.retain(|item| item.id != todo_id);
        if trash.items.len() == old_len {
            return Err(AppError::NotFound(format!("todo {todo_id}")));
        }
        Ok(vec![TRASH_CHECKLIST_ID.to_owned(), todo_id])
    })
    .await
}

#[tauri::command]
pub async fn purge_all_trash(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        let trash = snapshot.checklist_mut(TRASH_CHECKLIST_ID)?;
        if trash.items.is_empty() {
            return Err(AppError::Validation("回收站已经为空".to_owned()));
        }
        let mut changed = vec![TRASH_CHECKLIST_ID.to_owned()];
        changed.extend(trash.items.iter().map(|item| item.id.clone()));
        trash.items.clear();
        Ok(changed)
    })
    .await
}

#[tauri::command]
pub async fn set_sort_mode(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    sort_mode: SortMode,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        snapshot.sort_mode = sort_mode;
        Ok(vec!["sort-mode".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn set_hide_completed(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    hide_completed: bool,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        snapshot.hide_completed = hide_completed;
        Ok(vec!["hide-completed".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn set_quick_delete(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    quick_delete: bool,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        snapshot.quick_delete = quick_delete;
        Ok(vec!["quick-delete".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn set_deadline_countdown(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    visible: bool,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        snapshot.show_deadline_countdown = visible;
        Ok(vec!["deadline-countdown".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn update_settings(
    app: tauri::AppHandle,
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    settings: AppSettings,
) -> Result<MutationResult, AppError> {
    ensure_revision(&state, expected_revision).await?;
    let previous_autostart = state.inner.lock().await.snapshot.settings.autostart_enabled;
    if previous_autostart != settings.autostart_enabled {
        let result = if settings.autostart_enabled {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };
        result.map_err(|error| AppError::Platform(error.to_string()))?;
    }
    mutate(state, expected_revision, |snapshot| {
        snapshot.settings = settings.normalized();
        Ok(vec![SETTINGS_CHECKLIST_ID.to_owned()])
    })
    .await
}

fn validated_checklist_name(
    snapshot: &AppSnapshot,
    input: &str,
    editing_id: Option<&str>,
) -> Result<String, AppError> {
    let name = input.trim();
    if name.is_empty() {
        return Err(AppError::Validation("清单名不能为空".to_owned()));
    }
    if name.eq_ignore_ascii_case("TRASH") || name.eq_ignore_ascii_case("SETTINGS") {
        return Err(AppError::Validation(
            "清单名不能使用特殊页面名称".to_owned(),
        ));
    }
    if snapshot
        .checklists
        .iter()
        .any(|list| Some(list.id.as_str()) != editing_id && list.name.eq_ignore_ascii_case(name))
    {
        return Err(AppError::Validation("清单名已存在".to_owned()));
    }
    Ok(name.to_owned())
}
