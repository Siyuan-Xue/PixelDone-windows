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

#[derive(Clone, Debug, Default)]
pub struct ReminderDeliveryState {
    pub scheduled_at_millis: Option<i64>,
    pub snoozed_until_millis: Option<i64>,
    pub last_fired_at_millis: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalTodoAttachment {
    pub todo_id: String,
    pub local_file_name: Option<String>,
    pub attachment_id: Option<String>,
    pub object_path: Option<String>,
    pub content_sha256: Option<String>,
    pub content_type: Option<String>,
    pub byte_size: Option<i64>,
    pub updated_at_millis: i64,
    pub deleted_at_millis: Option<i64>,
    pub remote_version: Option<i64>,
    pub sync_state: String,
    pub last_error: Option<String>,
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

    pub async fn reminder_delivery_state(
        &self,
        todo_id: &str,
    ) -> Result<ReminderDeliveryState, AppError> {
        let row = sqlx::query(
            "SELECT scheduled_at_millis, snoozed_until_millis, last_fired_at_millis FROM reminder_delivery_state WHERE todo_id = ?",
        )
        .bind(todo_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(|row| ReminderDeliveryState {
                scheduled_at_millis: row.try_get("scheduled_at_millis").ok(),
                snoozed_until_millis: row.try_get("snoozed_until_millis").ok(),
                last_fired_at_millis: row.try_get("last_fired_at_millis").ok(),
            })
            .unwrap_or_default())
    }

    pub async fn save_reminder_delivery_state(
        &self,
        todo_id: &str,
        scheduled_at_millis: Option<i64>,
        snoozed_until_millis: Option<i64>,
        last_fired_at_millis: Option<i64>,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO reminder_delivery_state (todo_id, scheduled_at_millis, snoozed_until_millis, last_fired_at_millis, updated_at_millis) VALUES (?, ?, ?, ?, ?) ON CONFLICT(todo_id) DO UPDATE SET scheduled_at_millis = excluded.scheduled_at_millis, snoozed_until_millis = excluded.snoozed_until_millis, last_fired_at_millis = excluded.last_fired_at_millis, updated_at_millis = excluded.updated_at_millis",
        )
        .bind(todo_id)
        .bind(scheduled_at_millis)
        .bind(snoozed_until_millis)
        .bind(last_fired_at_millis)
        .bind(crate::domain::unix_now_millis())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn attachment(&self, todo_id: &str) -> Result<Option<LocalTodoAttachment>, AppError> {
        let row = sqlx::query(
            "SELECT todo_id, local_file_name, attachment_id, object_path, content_sha256, content_type, byte_size, updated_at_millis, deleted_at_millis, remote_version, sync_state, last_error FROM todo_attachments WHERE todo_id = ?",
        )
        .bind(todo_id)
        .fetch_optional(&self.pool)
        .await?;
        row.map(read_attachment).transpose()
    }

    pub async fn attachments(&self) -> Result<Vec<LocalTodoAttachment>, AppError> {
        sqlx::query(
            "SELECT todo_id, local_file_name, attachment_id, object_path, content_sha256, content_type, byte_size, updated_at_millis, deleted_at_millis, remote_version, sync_state, last_error FROM todo_attachments ORDER BY updated_at_millis, todo_id",
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(read_attachment)
        .collect()
    }

    pub async fn save_attachment(&self, value: &LocalTodoAttachment) -> Result<(), AppError> {
        sqlx::query(
            "INSERT INTO todo_attachments (todo_id, local_file_name, attachment_id, object_path, content_sha256, content_type, byte_size, updated_at_millis, deleted_at_millis, remote_version, sync_state, last_error) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(todo_id) DO UPDATE SET local_file_name = excluded.local_file_name, attachment_id = excluded.attachment_id, object_path = excluded.object_path, content_sha256 = excluded.content_sha256, content_type = excluded.content_type, byte_size = excluded.byte_size, updated_at_millis = excluded.updated_at_millis, deleted_at_millis = excluded.deleted_at_millis, remote_version = excluded.remote_version, sync_state = excluded.sync_state, last_error = excluded.last_error",
        )
        .bind(&value.todo_id)
        .bind(&value.local_file_name)
        .bind(&value.attachment_id)
        .bind(&value.object_path)
        .bind(&value.content_sha256)
        .bind(&value.content_type)
        .bind(value.byte_size)
        .bind(value.updated_at_millis)
        .bind(value.deleted_at_millis)
        .bind(value.remote_version)
        .bind(&value.sync_state)
        .bind(&value.last_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_attachment(&self, todo_id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM todo_attachments WHERE todo_id = ?")
            .bind(todo_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn queue_local_image_cleanup(
        &self,
        todo_id: &str,
        object_path: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "INSERT OR IGNORE INTO todo_image_local_cleanup_queue (todo_id, object_path, queued_at_millis) VALUES (?, ?, ?)",
        )
        .bind(todo_id)
        .bind(object_path)
        .bind(crate::domain::unix_now_millis())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn local_image_cleanup_paths(&self) -> Result<Vec<(String, String)>, AppError> {
        let rows = sqlx::query(
            "SELECT todo_id, object_path FROM todo_image_local_cleanup_queue ORDER BY queued_at_millis, object_path",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| Ok((row.try_get("todo_id")?, row.try_get("object_path")?)))
            .collect()
    }

    pub async fn delete_local_image_cleanup_path(
        &self,
        todo_id: &str,
        object_path: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            "DELETE FROM todo_image_local_cleanup_queue WHERE todo_id = ? AND object_path = ?",
        )
        .bind(todo_id)
        .bind(object_path)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn snoozed_reminders(&self, now_millis: i64) -> Result<Vec<(String, i64)>, AppError> {
        let rows = sqlx::query(
            "SELECT todo_id, snoozed_until_millis FROM reminder_delivery_state WHERE snoozed_until_millis > ?",
        )
        .bind(now_millis)
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok((
                    row.try_get("todo_id")?,
                    row.try_get("snoozed_until_millis")?,
                ))
            })
            .collect()
    }

    pub async fn delete_orphaned_reminder_delivery_states(
        &self,
        active_todo_ids: &[String],
    ) -> Result<(), AppError> {
        if active_todo_ids.is_empty() {
            sqlx::query("DELETE FROM reminder_delivery_state")
                .execute(&self.pool)
                .await?;
            return Ok(());
        }
        let placeholders = std::iter::repeat_n("?", active_todo_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let statement =
            format!("DELETE FROM reminder_delivery_state WHERE todo_id NOT IN ({placeholders})");
        let mut query = sqlx::query(&statement);
        for id in active_todo_ids {
            query = query.bind(id);
        }
        query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn autostart_initialized(&self) -> Result<bool, AppError> {
        let initialized = sqlx::query_scalar::<_, i64>(
            "SELECT autostart_initialized FROM local_settings WHERE id = 'settings'",
        )
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or_default();
        Ok(initialized != 0)
    }

    pub async fn mark_autostart_initialized(&self) -> Result<(), AppError> {
        sqlx::query("UPDATE local_settings SET autostart_initialized = 1 WHERE id = 'settings'")
            .execute(&self.pool)
            .await?;
        Ok(())
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
        let (last_update_check_at_millis, next_update_check_at_millis) =
            self.load_update_timing().await?;
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
                    last_checked_at_millis: last_update_check_at_millis,
                    next_check_at_millis: next_update_check_at_millis,
                    ..UpdateView::default()
                },
            }
            .normalized(),
        ))
    }

    async fn load_settings(&self) -> Result<AppSettings, AppError> {
        let Some(row) = sqlx::query(
            "SELECT dark_theme, dock_plus_placement, dock_actions_json, never_show_update_dialog, future_sync_enabled, language_mode, autostart_enabled, automatic_update_check_enabled, enhanced_xhigh_alarm_enabled, sidebar_width_px FROM local_settings WHERE id = 'settings'",
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
            autostart_enabled: row.try_get::<i64, _>("autostart_enabled")? != 0,
            automatic_update_check_enabled: row
                .try_get::<i64, _>("automatic_update_check_enabled")?
                != 0,
            enhanced_xhigh_alarm_enabled: row.try_get::<i64, _>("enhanced_xhigh_alarm_enabled")?
                != 0,
            sidebar_width_px: row.try_get("sidebar_width_px")?,
        }
        .normalized())
    }

    async fn load_update_timing(&self) -> Result<(Option<i64>, Option<i64>), AppError> {
        let row = sqlx::query(
            "SELECT last_update_check_at_millis, next_update_check_at_millis FROM local_settings WHERE id = 'settings'",
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(row
            .map(|row| {
                (
                    row.try_get("last_update_check_at_millis").unwrap_or(None),
                    row.try_get("next_update_check_at_millis").unwrap_or(None),
                )
            })
            .unwrap_or((None, None)))
    }

    pub async fn save_update_timing(
        &self,
        last_checked_at_millis: i64,
        next_check_at_millis: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE local_settings SET last_update_check_at_millis = ?, next_update_check_at_millis = ? WHERE id = 'settings'",
        )
        .bind(last_checked_at_millis)
        .bind(next_check_at_millis)
        .execute(&self.pool)
        .await?;
        Ok(())
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
            "INSERT INTO local_settings (id, dark_theme, dock_plus_placement, dock_actions_json, never_show_update_dialog, future_sync_enabled, language_mode, autostart_enabled, automatic_update_check_enabled, enhanced_xhigh_alarm_enabled, sidebar_width_px) VALUES ('settings', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET dark_theme = excluded.dark_theme, dock_plus_placement = excluded.dock_plus_placement, dock_actions_json = excluded.dock_actions_json, never_show_update_dialog = excluded.never_show_update_dialog, future_sync_enabled = excluded.future_sync_enabled, language_mode = excluded.language_mode, autostart_enabled = excluded.autostart_enabled, automatic_update_check_enabled = excluded.automatic_update_check_enabled, enhanced_xhigh_alarm_enabled = excluded.enhanced_xhigh_alarm_enabled, sidebar_width_px = excluded.sidebar_width_px",
        )
        .bind(i64::from(snapshot.settings.dark_theme))
        .bind(placement_name(snapshot.settings.dock.plus_placement))
        .bind(dock_actions)
        .bind(i64::from(snapshot.settings.never_show_update_dialog))
        .bind(i64::from(snapshot.settings.future_sync_enabled))
        .bind(snapshot.settings.language_mode.sync_value())
        .bind(i64::from(snapshot.settings.autostart_enabled))
        .bind(i64::from(
            snapshot.settings.automatic_update_check_enabled,
        ))
        .bind(i64::from(
            snapshot.settings.enhanced_xhigh_alarm_enabled,
        ))
        .bind(snapshot.settings.sidebar_width_px)
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
            "INSERT INTO sync_cursors (owner_user_id, remote_version, schema_version, updated_at_millis) VALUES (?, ?, 32, ?) ON CONFLICT(owner_user_id) DO UPDATE SET remote_version = excluded.remote_version, schema_version = excluded.schema_version, updated_at_millis = excluded.updated_at_millis",
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

fn read_attachment(row: sqlx::sqlite::SqliteRow) -> Result<LocalTodoAttachment, AppError> {
    Ok(LocalTodoAttachment {
        todo_id: row.try_get("todo_id")?,
        local_file_name: row.try_get("local_file_name")?,
        attachment_id: row.try_get("attachment_id")?,
        object_path: row.try_get("object_path")?,
        content_sha256: row.try_get("content_sha256")?,
        content_type: row.try_get("content_type")?,
        byte_size: row.try_get("byte_size")?,
        updated_at_millis: row.try_get("updated_at_millis")?,
        deleted_at_millis: row.try_get("deleted_at_millis")?,
        remote_version: row.try_get("remote_version")?,
        sync_state: row.try_get("sync_state")?,
        last_error: row.try_get("last_error")?,
    })
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
