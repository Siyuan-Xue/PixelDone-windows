use std::path::Path;

use sqlx::{Row, SqlitePool, sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions};

use crate::domain::{
    AppError, AppSettings, AppSnapshot, Checklist, ChecklistKind, DockAction, DockConfig,
    DockPlusPlacement, ReminderRepeat, SortMode, TodoItem, TodoPriority,
};

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
            "SELECT selected_checklist_id, revision, sort_mode, hide_completed, quick_delete FROM app_metadata WHERE id = 'app'",
        )
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        let checklist_rows = sqlx::query(
            "SELECT id, name, created_at_millis FROM checklists WHERE deleted_at_millis IS NULL ORDER BY sort_index",
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
                "SELECT id, title, priority, due_at_millis, completed, created_at_millis, reminder_repeat, image_file_name, trashed_from_checklist_id, trashed_from_checklist_name, trashed_at_millis FROM todo_items WHERE checklist_id = ? ORDER BY sort_index",
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
                    reminder_repeat: parse_repeat(&item.try_get::<String, _>("reminder_repeat")?)?,
                    image_file_name: item.try_get("image_file_name")?,
                    trashed_from_checklist_id: item.try_get("trashed_from_checklist_id")?,
                    trashed_from_checklist_name: item.try_get("trashed_from_checklist_name")?,
                    trashed_at_millis: item.try_get("trashed_at_millis")?,
                });
            }
            checklists.push(Checklist {
                id,
                name: row.try_get("name")?,
                kind,
                items,
                created_at_millis: row.try_get("created_at_millis")?,
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
                settings,
            }
            .normalized(),
        ))
    }

    async fn load_settings(&self) -> Result<AppSettings, AppError> {
        let Some(row) = sqlx::query(
            "SELECT dark_theme, dock_plus_placement, dock_actions_json, never_show_update_dialog, future_sync_enabled FROM local_settings WHERE id = 'settings'",
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
                "INSERT INTO checklists (id, name, sort_index, created_at_millis, deleted_at_millis) VALUES (?, ?, ?, ?, NULL)",
            )
            .bind(&checklist.id)
            .bind(&checklist.name)
            .bind(list_index as i64)
            .bind(checklist.created_at_millis)
            .execute(&mut *transaction)
            .await?;

            for (item_index, item) in checklist.items.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO todo_items (id, checklist_id, sort_index, title, priority, due_at_millis, completed, created_at_millis, reminder_repeat, image_file_name, trashed_from_checklist_id, trashed_from_checklist_name, trashed_at_millis) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&item.id)
                .bind(&checklist.id)
                .bind(item_index as i64)
                .bind(&item.title)
                .bind(priority_name(item.priority))
                .bind(item.due_at_millis)
                .bind(i64::from(item.completed))
                .bind(item.created_at_millis)
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
            "INSERT INTO app_metadata (id, selected_checklist_id, revision, sort_mode, hide_completed, quick_delete) VALUES ('app', ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET selected_checklist_id = excluded.selected_checklist_id, revision = excluded.revision, sort_mode = excluded.sort_mode, hide_completed = excluded.hide_completed, quick_delete = excluded.quick_delete",
        )
        .bind(&snapshot.selected_checklist_id)
        .bind(snapshot.revision)
        .bind(sort_mode_name(snapshot.sort_mode))
        .bind(i64::from(snapshot.hide_completed))
        .bind(i64::from(snapshot.quick_delete))
        .execute(&mut *transaction)
        .await?;

        let dock_actions = serde_json::to_string(&snapshot.settings.dock.actions)
            .map_err(|error| AppError::Database(error.to_string()))?;
        sqlx::query(
            "INSERT INTO local_settings (id, dark_theme, dock_plus_placement, dock_actions_json, never_show_update_dialog, future_sync_enabled) VALUES ('settings', ?, ?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET dark_theme = excluded.dark_theme, dock_plus_placement = excluded.dock_plus_placement, dock_actions_json = excluded.dock_actions_json, never_show_update_dialog = excluded.never_show_update_dialog, future_sync_enabled = excluded.future_sync_enabled",
        )
        .bind(i64::from(snapshot.settings.dark_theme))
        .bind(placement_name(snapshot.settings.dock.plus_placement))
        .bind(dock_actions)
        .bind(i64::from(snapshot.settings.never_show_update_dialog))
        .bind(i64::from(snapshot.settings.future_sync_enabled))
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
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
