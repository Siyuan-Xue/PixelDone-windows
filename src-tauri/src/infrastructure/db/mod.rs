use std::path::Path;

use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions};

use crate::domain::{
    AppError, AppLanguage, AppSettings, AppSnapshot, AuthView, Checklist, ChecklistKind,
    DockAction, DockConfig, DockPlusPlacement, ReminderRepeat, ReminderRunView, SortMode,
    SyncRunView, SyncState, TodoItem, TodoPriority, UpdateView,
};

#[derive(Clone, Debug)]
pub struct DirtyRecord {
    pub record_type: String,
    pub local_id: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTombstone {
    pub record_type: String,
    pub local_id: String,
    pub deleted_at_millis: i64,
    pub remote_version: Option<i64>,
}

#[derive(Clone, Debug)]
pub struct StoredConflict {
    pub record_type: String,
    pub local_id: String,
    pub local_payload_json: String,
    pub remote_payload_json: String,
    pub fields_json: String,
    pub message: String,
    pub remote_version: Option<i64>,
}

#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub async fn open(path: &Path) -> Result<Self, AppError> {
        let options = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .foreign_keys(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await?;
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|error| AppError::Database(error.to_string()))?;
        Ok(Self { pool })
    }

    pub async fn load_snapshot(&self) -> Result<Option<AppSnapshot>, AppError> {
        let Some(metadata) = sqlx::query(
            "SELECT selected_checklist_id, revision, sort_mode, hide_completed, quick_delete, show_deadline_countdown FROM app_metadata WHERE id = 'app'",
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let checklist_rows = sqlx::query(
            "SELECT id, name, created_at_millis, updated_at_millis, remote_version FROM checklists WHERE deleted_at_millis IS NULL ORDER BY sort_index",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut checklists = Vec::with_capacity(checklist_rows.len());
        for row in checklist_rows {
            let id: String = row.try_get("id")?;
            let kind = match id.as_str() {
                "trash" => ChecklistKind::Trash,
                "settings" => ChecklistKind::Settings,
                _ => ChecklistKind::Normal,
            };
            let item_rows = sqlx::query(
                "SELECT id, title, priority, due_at_millis, completed, created_at_millis, updated_at_millis, remote_version, reminder_repeat, image_file_name, trashed_from_checklist_id, trashed_from_checklist_name, trashed_at_millis FROM todo_items WHERE checklist_id = ? ORDER BY sort_index",
            )
            .bind(&id)
            .fetch_all(&self.pool)
            .await?;
            let mut items = Vec::with_capacity(item_rows.len());
            for item in item_rows {
                items.push(TodoItem {
                    id: item.try_get("id")?,
                    title: item.try_get("title")?,
                    priority: parse_priority(&item.try_get::<String, _>("priority")?)?,
                    due_at_millis: item.try_get("due_at_millis")?,
                    completed: item.try_get::<i64, _>("completed")? != 0,
                    created_at_millis: item.try_get("created_at_millis")?,
                    updated_at_millis: item.try_get("updated_at_millis")?,
                    reminder_repeat: parse_repeat(&item.try_get::<String, _>("reminder_repeat")?)?,
                    image_file_name: item.try_get("image_file_name")?,
                    trashed_from_checklist_id: item.try_get("trashed_from_checklist_id")?,
                    trashed_from_checklist_name: item.try_get("trashed_from_checklist_name")?,
                    trashed_at_millis: item.try_get("trashed_at_millis")?,
                    remote_version: item.try_get("remote_version")?,
                });
            }
            checklists.push(Checklist {
                id,
                name: row.try_get("name")?,
                kind,
                items,
                created_at_millis: row.try_get("created_at_millis")?,
                updated_at_millis: row.try_get("updated_at_millis")?,
                remote_version: row.try_get("remote_version")?,
            });
        }

        let settings = self.load_settings().await?;
        Ok(Some(
            AppSnapshot {
                revision: metadata.try_get("revision")?,
                checklists,
                selected_checklist_id: metadata.try_get("selected_checklist_id")?,
                sort_mode: parse_sort_mode(&metadata.try_get::<String, _>("sort_mode")?)?,
                hide_completed: metadata.try_get::<i64, _>("hide_completed")? != 0,
                quick_delete: metadata.try_get::<i64, _>("quick_delete")? != 0,
                show_deadline_countdown: metadata.try_get::<i64, _>("show_deadline_countdown")?
                    != 0,
                checklist_history: Vec::new(),
                settings,
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
            .normalized(),
        ))
    }

    async fn load_settings(&self) -> Result<AppSettings, AppError> {
        let Some(row) = sqlx::query(
            "SELECT dark_theme, dock_plus_placement, dock_actions_json, never_show_update_dialog, future_sync_enabled, language_mode FROM local_settings WHERE id = 'settings'",
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(AppSettings::default());
        };
        let actions_json: String = row.try_get("dock_actions_json")?;
        let actions = serde_json::from_str::<Vec<DockAction>>(&actions_json)
            .map_err(|error| AppError::Database(error.to_string()))?;
        Ok(AppSettings {
            dark_theme: row.try_get::<i64, _>("dark_theme")? != 0,
            dock: DockConfig {
                plus_placement: parse_placement(&row.try_get::<String, _>("dock_plus_placement")?)?,
                actions,
            }
            .normalized(),
            never_show_update_dialog: row.try_get::<i64, _>("never_show_update_dialog")? != 0,
            future_sync_enabled: row.try_get::<i64, _>("future_sync_enabled")? != 0,
            language_mode: AppLanguage::from_sync_value(
                &row.try_get::<String, _>("language_mode")?,
            ),
        })
    }

    pub async fn save_snapshot(&self, snapshot: &AppSnapshot) -> Result<(), AppError> {
        let mut transaction = self.pool.begin().await?;
        sqlx::query("DELETE FROM todo_items")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("DELETE FROM checklists")
            .execute(&mut *transaction)
            .await?;

        for (list_index, checklist) in snapshot.checklists.iter().enumerate() {
            sqlx::query(
                "INSERT INTO checklists (id, name, sort_index, created_at_millis, updated_at_millis, remote_version, deleted_at_millis) VALUES (?, ?, ?, ?, ?, ?, NULL)",
            )
            .bind(&checklist.id)
            .bind(&checklist.name)
            .bind(list_index as i64)
            .bind(checklist.created_at_millis)
            .bind(checklist.updated_at_millis)
            .bind(checklist.remote_version)
            .execute(&mut *transaction)
            .await?;

            for (item_index, item) in checklist.items.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO todo_items (id, checklist_id, sort_index, title, priority, due_at_millis, completed, created_at_millis, updated_at_millis, remote_version, reminder_repeat, image_file_name, trashed_from_checklist_id, trashed_from_checklist_name, trashed_at_millis) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&item.id)
                .bind(&checklist.id)
                .bind(item_index as i64)
                .bind(&item.title)
                .bind(priority_name(item.priority))
                .bind(item.due_at_millis)
                .bind(i64::from(item.completed))
                .bind(item.created_at_millis)
                .bind(item.updated_at_millis)
                .bind(item.remote_version)
                .bind(repeat_name(item.reminder_repeat))
                .bind(&item.image_file_name)
                .bind(&item.trashed_from_checklist_id)
                .bind(&item.trashed_from_checklist_name)
                .bind(item.trashed_at_millis)
                .execute(&mut *transaction)
                .await?;
            }
        }

        sqlx::query(
            "INSERT INTO app_metadata (id, selected_checklist_id, revision, sort_mode, hide_completed, quick_delete, show_deadline_countdown) VALUES ('app', ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET selected_checklist_id = excluded.selected_checklist_id, revision = excluded.revision, sort_mode = excluded.sort_mode, hide_completed = excluded.hide_completed, quick_delete = excluded.quick_delete, show_deadline_countdown = excluded.show_deadline_countdown",
        )
        .bind(&snapshot.selected_checklist_id)
        .bind(snapshot.revision)
        .bind(sort_mode_name(snapshot.sort_mode))
        .bind(i64::from(snapshot.hide_completed))
        .bind(i64::from(snapshot.quick_delete))
        .bind(i64::from(snapshot.show_deadline_countdown))
        .execute(&mut *transaction)
        .await?;

        let dock_actions = serde_json::to_string(&snapshot.settings.dock.actions)
            .map_err(|error| AppError::Database(error.to_string()))?;
        sqlx::query(
            "INSERT INTO local_settings (id, dark_theme, dock_plus_placement, dock_actions_json, never_show_update_dialog, future_sync_enabled, language_mode) VALUES ('settings', ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET dark_theme = excluded.dark_theme, dock_plus_placement = excluded.dock_plus_placement, dock_actions_json = excluded.dock_actions_json, never_show_update_dialog = excluded.never_show_update_dialog, future_sync_enabled = excluded.future_sync_enabled, language_mode = excluded.language_mode",
        )
        .bind(i64::from(snapshot.settings.dark_theme))
        .bind(placement_name(snapshot.settings.dock.plus_placement))
        .bind(dock_actions)
        .bind(i64::from(snapshot.settings.never_show_update_dialog))
        .bind(i64::from(snapshot.settings.future_sync_enabled))
        .bind(snapshot.settings.language_mode.sync_value())
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn save_snapshot_with_changes(
        &self,
        before: &AppSnapshot,
        after: &AppSnapshot,
    ) -> Result<(), AppError> {
        self.save_snapshot(after).await?;
        self.record_local_changes(before, after).await
    }

    async fn record_local_changes(
        &self,
        before: &AppSnapshot,
        after: &AppSnapshot,
    ) -> Result<(), AppError> {
        let before_lists = before
            .checklists
            .iter()
            .filter(|list| list.kind == ChecklistKind::Normal)
            .map(|list| (list.id.as_str(), list))
            .collect::<std::collections::HashMap<_, _>>();
        let after_lists = after
            .checklists
            .iter()
            .filter(|list| list.kind == ChecklistKind::Normal)
            .map(|list| (list.id.as_str(), list))
            .collect::<std::collections::HashMap<_, _>>();
        let before_items = before
            .checklists
            .iter()
            .flat_map(|list| list.items.iter())
            .map(|item| (item.id.as_str(), item))
            .collect::<std::collections::HashMap<_, _>>();
        let after_items = after
            .checklists
            .iter()
            .flat_map(|list| list.items.iter())
            .map(|item| (item.id.as_str(), item))
            .collect::<std::collections::HashMap<_, _>>();

        let mut transaction = self.pool.begin().await?;
        for (id, list) in &after_lists {
            if before_lists.get(id).copied() != Some(*list) {
                mark_dirty(&mut transaction, "checklist", id).await?;
            }
        }
        for (id, item) in &after_items {
            if before_items.get(id).copied() != Some(*item) {
                mark_dirty(&mut transaction, "item", id).await?;
            }
        }
        let now = crate::domain::unix_now_millis();
        for (id, list) in &before_lists {
            if !after_lists.contains_key(id) {
                upsert_tombstone(&mut transaction, "checklist", id, now, list.remote_version)
                    .await?;
            }
        }
        for (id, item) in &before_items {
            if !after_items.contains_key(id) {
                upsert_tombstone(&mut transaction, "item", id, now, item.remote_version).await?;
            }
        }
        if before.settings.language_mode != after.settings.language_mode {
            mark_dirty(&mut transaction, "settings", "language").await?;
        }
        transaction.commit().await?;
        Ok(())
    }

    pub async fn dirty_records(&self) -> Result<Vec<DirtyRecord>, AppError> {
        let rows = sqlx::query(
            "SELECT record_type, local_id FROM sync_dirty_records ORDER BY record_type, local_id",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(DirtyRecord {
                    record_type: row.try_get("record_type")?,
                    local_id: row.try_get("local_id")?,
                })
            })
            .collect()
    }

    pub async fn clear_dirty(&self, record_type: &str, local_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sync_dirty_records WHERE record_type = ? AND local_id = ?")
            .bind(record_type)
            .bind(local_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn tombstones(&self) -> Result<Vec<LocalTombstone>, AppError> {
        let rows = sqlx::query(
            "SELECT record_type, local_id, deleted_at_millis, remote_version FROM sync_tombstones ORDER BY deleted_at_millis",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(LocalTombstone {
                    record_type: row.try_get("record_type")?,
                    local_id: row.try_get("local_id")?,
                    deleted_at_millis: row.try_get("deleted_at_millis")?,
                    remote_version: row.try_get("remote_version")?,
                })
            })
            .collect()
    }

    pub async fn delete_tombstone(
        &self,
        record_type: &str,
        local_id: &str,
    ) -> Result<(), AppError> {
        sqlx::query("DELETE FROM sync_tombstones WHERE record_type = ? AND local_id = ?")
            .bind(record_type)
            .bind(local_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn sync_cursor(&self, owner: &str) -> Result<i64, AppError> {
        Ok(sqlx::query_scalar::<_, i64>(
            "SELECT remote_version FROM sync_cursors WHERE owner_user_id = ?",
        )
        .bind(owner)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or(0))
    }

    pub async fn save_sync_cursor(&self, owner: &str, version: i64) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sync_cursors (owner_user_id, remote_version, schema_version, updated_at_millis) VALUES (?, ?, 31, ?) ON CONFLICT(owner_user_id) DO UPDATE SET remote_version = excluded.remote_version, schema_version = excluded.schema_version, updated_at_millis = excluded.updated_at_millis",
        )
        .bind(owner)
        .bind(version)
        .bind(crate::domain::unix_now_millis())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn language_remote_version(&self) -> Result<Option<i64>, AppError> {
        Ok(sqlx::query_scalar::<_, Option<i64>>(
            "SELECT language_remote_version FROM local_settings WHERE id = 'settings'",
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten())
    }

    pub async fn set_language_remote_version(&self, version: Option<i64>) -> Result<(), AppError> {
        sqlx::query("UPDATE local_settings SET language_remote_version = ? WHERE id = 'settings'")
            .bind(version)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn save_conflict(&self, conflict: &StoredConflict) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO sync_conflicts (owner_user_id, record_type, local_id, local_payload_json, remote_payload_json, fields_json, remote_version, created_at_millis) VALUES ('current', ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(owner_user_id, record_type, local_id) DO UPDATE SET local_payload_json = excluded.local_payload_json, remote_payload_json = excluded.remote_payload_json, fields_json = excluded.fields_json, remote_version = excluded.remote_version, created_at_millis = excluded.created_at_millis",
        )
        .bind(&conflict.record_type)
        .bind(&conflict.local_id)
        .bind(&conflict.local_payload_json)
        .bind(&conflict.remote_payload_json)
        .bind(&conflict.fields_json)
        .bind(conflict.remote_version)
        .bind(crate::domain::unix_now_millis())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn conflicts(&self) -> Result<Vec<StoredConflict>, AppError> {
        let rows = sqlx::query(
            "SELECT record_type, local_id, local_payload_json, remote_payload_json, fields_json, remote_version FROM sync_conflicts WHERE owner_user_id = 'current' ORDER BY created_at_millis",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(StoredConflict {
                    record_type: row.try_get("record_type")?,
                    local_id: row.try_get("local_id")?,
                    local_payload_json: row.try_get("local_payload_json")?,
                    remote_payload_json: row.try_get("remote_payload_json")?,
                    fields_json: row.try_get("fields_json")?,
                    message: "remote_version_changed".to_owned(),
                    remote_version: row.try_get("remote_version")?,
                })
            })
            .collect()
    }

    pub async fn delete_conflict(&self, record_type: &str, local_id: &str) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM sync_conflicts WHERE owner_user_id = 'current' AND record_type = ? AND local_id = ?",
        )
        .bind(record_type)
        .bind(local_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}

async fn mark_dirty(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    record_type: &str,
    local_id: &str,
) -> Result<(), AppError> {
    sqlx::query("INSERT OR IGNORE INTO sync_dirty_records (record_type, local_id) VALUES (?, ?)")
        .bind(record_type)
        .bind(local_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn upsert_tombstone(
    transaction: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    record_type: &str,
    local_id: &str,
    deleted_at_millis: i64,
    remote_version: Option<i64>,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO sync_tombstones (record_type, local_id, deleted_at_millis, remote_version) VALUES (?, ?, ?, ?) ON CONFLICT(record_type, local_id) DO UPDATE SET deleted_at_millis = excluded.deleted_at_millis, remote_version = COALESCE(sync_tombstones.remote_version, excluded.remote_version)",
    )
    .bind(record_type)
    .bind(local_id)
    .bind(deleted_at_millis)
    .bind(remote_version)
    .execute(&mut **transaction)
    .await?;
    sqlx::query("DELETE FROM sync_dirty_records WHERE record_type = ? AND local_id = ?")
        .bind(record_type)
        .bind(local_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

fn parse_priority(value: &str) -> Result<TodoPriority, AppError> {
    match value {
        "XHIGH" => Ok(TodoPriority::Xhigh),
        "HIGH" => Ok(TodoPriority::High),
        "MEDIUM" => Ok(TodoPriority::Medium),
        "LOW" => Ok(TodoPriority::Low),
        other => Err(AppError::Database(format!("unknown priority {other}"))),
    }
}

fn priority_name(value: TodoPriority) -> &'static str {
    match value {
        TodoPriority::Xhigh => "XHIGH",
        TodoPriority::High => "HIGH",
        TodoPriority::Medium => "MEDIUM",
        TodoPriority::Low => "LOW",
    }
}

fn parse_repeat(value: &str) -> Result<ReminderRepeat, AppError> {
    match value {
        "NONE" => Ok(ReminderRepeat::None),
        "DAILY" => Ok(ReminderRepeat::Daily),
        "WEEKLY" => Ok(ReminderRepeat::Weekly),
        other => Err(AppError::Database(format!("unknown repeat {other}"))),
    }
}

fn repeat_name(value: ReminderRepeat) -> &'static str {
    match value {
        ReminderRepeat::None => "NONE",
        ReminderRepeat::Daily => "DAILY",
        ReminderRepeat::Weekly => "WEEKLY",
    }
}

fn parse_sort_mode(value: &str) -> Result<SortMode, AppError> {
    match value {
        "PRIORITY" => Ok(SortMode::Priority),
        "TIME" => Ok(SortMode::Time),
        other => Err(AppError::Database(format!("unknown sort mode {other}"))),
    }
}

fn sort_mode_name(value: SortMode) -> &'static str {
    match value {
        SortMode::Priority => "PRIORITY",
        SortMode::Time => "TIME",
    }
}

fn parse_placement(value: &str) -> Result<DockPlusPlacement, AppError> {
    match value {
        "CENTER" => Ok(DockPlusPlacement::Center),
        "LEFT_EDGE" => Ok(DockPlusPlacement::LeftEdge),
        "RIGHT_EDGE" => Ok(DockPlusPlacement::RightEdge),
        other => Err(AppError::Database(format!(
            "unknown dock placement {other}"
        ))),
    }
}

fn placement_name(value: DockPlusPlacement) -> &'static str {
    match value {
        DockPlusPlacement::Center => "CENTER",
        DockPlusPlacement::LeftEdge => "LEFT_EDGE",
        DockPlusPlacement::RightEdge => "RIGHT_EDGE",
    }
}
