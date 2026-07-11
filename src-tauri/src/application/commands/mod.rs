use tauri::State;
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
pub mod sync;
pub mod update;

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
        let target_id = item
            .trashed_from_checklist_id
            .clone()
            .filter(|id| id != TRASH_CHECKLIST_ID && id != SETTINGS_CHECKLIST_ID)
            .or_else(|| {
                snapshot
                    .checklists
                    .iter()
                    .find(|list| list.kind == ChecklistKind::Normal)
                    .map(|list| list.id.clone())
            })
            .unwrap_or_else(|| "main".to_owned());

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
                Checklist::new_normal(target_id.clone(), name, unix_now_millis()),
            );
        }
        item.trashed_from_checklist_id = None;
        item.trashed_from_checklist_name = None;
        item.trashed_at_millis = None;
        item.updated_at_millis = unix_now_millis();
        snapshot.checklist_mut(&target_id)?.items.push(item);
        Ok(vec![TRASH_CHECKLIST_ID.to_owned(), target_id, todo_id])
    })
    .await
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
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    settings: AppSettings,
) -> Result<MutationResult, AppError> {
    mutate(state, expected_revision, |snapshot| {
        snapshot.settings = AppSettings {
            dock: settings.dock.normalized(),
            ..settings
        };
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
