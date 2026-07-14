use pixeldone_windows_lib::{
    domain::{AppSnapshot, DEFAULT_CHECKLIST_ID, ReminderRepeat, TodoItem, TodoPriority},
    infrastructure::{db::LocalTodoAttachment, repository::SqliteRepository},
};

use std::path::PathBuf;

#[test]
fn migrations_persist_and_reload_the_initial_snapshot() {
    tauri::async_runtime::block_on(async {
        let database_path = std::env::temp_dir().join(format!(
            "pixeldone-migration-{}.sqlite3",
            std::process::id()
        ));
        let repository = SqliteRepository::open(&database_path)
            .await
            .expect("migrations should apply");
        let mut snapshot = AppSnapshot::initial(42);
        assert!(!snapshot.settings.enhanced_xhigh_alarm_enabled);
        assert_eq!(snapshot.settings.sidebar_width_px, 320);
        snapshot.settings.enhanced_xhigh_alarm_enabled = true;
        snapshot.settings.sidebar_width_px = 520;
        repository
            .save_snapshot(&snapshot)
            .await
            .expect("snapshot should persist");
        repository
            .save_attachment(&LocalTodoAttachment {
                todo_id: "migration-image".to_owned(),
                local_file_name: Some("migration-image.png".to_owned()),
                updated_at_millis: 43,
                sync_state: "PENDING_UPLOAD".to_owned(),
                ..LocalTodoAttachment::default()
            })
            .await
            .expect("attachment metadata should persist");
        repository
            .save_snapshot(&snapshot)
            .await
            .expect("snapshot replacement should not cascade attachment metadata");
        let restored = repository
            .load_snapshot()
            .await
            .expect("snapshot should load")
            .expect("metadata row should exist");

        assert_eq!(restored, snapshot);
        assert_eq!(
            repository
                .attachment("migration-image")
                .await
                .expect("attachment should load")
                .expect("attachment should survive snapshot replacement")
                .local_file_name
                .as_deref(),
            Some("migration-image.png")
        );
        repository.close().await;
        let _ = std::fs::remove_file(database_path);
    });
}

#[test]
fn migration_nine_persists_pristine_and_mutation_state() {
    tauri::async_runtime::block_on(async {
        let database_path = std::env::temp_dir().join(format!(
            "pixeldone-migration-nine-{}.sqlite3",
            std::process::id()
        ));
        let repository = SqliteRepository::open(&database_path)
            .await
            .expect("migration nine should apply");

        assert!(!repository.pristine_initialized("owner").await.unwrap());
        repository.mark_pristine_initialized("owner").await.unwrap();
        repository
            .save_pristine_record("owner", "item", "todo", r#"{"title":"base"}"#, Some(7))
            .await
            .unwrap();
        repository
            .save_pending_mutation("owner", "mutation-1", 7, r#"{"items":[]}"#)
            .await
            .unwrap();

        assert!(repository.pristine_initialized("owner").await.unwrap());
        assert_eq!(
            repository
                .pristine_record("owner", "item", "todo")
                .await
                .unwrap()
                .unwrap()
                .remote_version,
            Some(7)
        );
        assert_eq!(
            repository
                .pending_mutation("owner")
                .await
                .unwrap()
                .unwrap()
                .mutation_uuid,
            "mutation-1"
        );

        repository.close().await;
        let _ = std::fs::remove_file(database_path);
    });
}

#[test]
fn editing_a_todo_does_not_dirty_its_parent_checklist() {
    tauri::async_runtime::block_on(async {
        let database_path = std::env::temp_dir().join(format!(
            "pixeldone-item-dirty-{}.sqlite3",
            std::process::id()
        ));
        let repository = SqliteRepository::open(&database_path).await.unwrap();
        let mut before = AppSnapshot::initial(42);
        before
            .checklists
            .iter_mut()
            .find(|list| list.id == DEFAULT_CHECKLIST_ID)
            .unwrap()
            .items
            .push(TodoItem {
                id: "todo".to_owned(),
                title: "Before".to_owned(),
                priority: TodoPriority::Medium,
                due_at_millis: 0,
                completed: false,
                created_at_millis: 42,
                updated_at_millis: 42,
                reminder_repeat: ReminderRepeat::None,
                image_file_name: None,
                trashed_from_checklist_id: None,
                trashed_from_checklist_name: None,
                trashed_at_millis: None,
                remote_version: Some(1),
            });
        repository.save_snapshot(&before).await.unwrap();
        let mut after = before.clone();
        let todo = &mut after
            .checklists
            .iter_mut()
            .find(|list| list.id == DEFAULT_CHECKLIST_ID)
            .unwrap()
            .items[0];
        todo.title = "After".to_owned();
        todo.updated_at_millis = 43;

        repository
            .save_snapshot_with_changes(&before, &after)
            .await
            .unwrap();
        let dirty = repository.dirty_records().await.unwrap();

        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].record_type, "item");
        assert_eq!(dirty[0].local_id, "todo");

        repository.close().await;
        let _ = std::fs::remove_file(database_path);
    });
}

#[test]
#[ignore = "requires a disposable copy of a deployed database"]
fn deployed_database_copy_upgrades_without_checksum_drift() {
    let database_path = PathBuf::from(
        std::env::var_os("PIXELDONE_MIGRATION_COMPAT_DATABASE")
            .expect("PIXELDONE_MIGRATION_COMPAT_DATABASE must point to a disposable database copy"),
    );
    tauri::async_runtime::block_on(async {
        let repository = SqliteRepository::open(&database_path)
            .await
            .expect("the deployed database copy should upgrade without checksum drift");
        let snapshot = repository
            .load_snapshot()
            .await
            .expect("the upgraded snapshot should load")
            .expect("the deployed database should contain a snapshot");
        assert!((200..=560).contains(&snapshot.settings.sidebar_width_px));
        repository.close().await;
    });
}
