use pixeldone_windows_lib::{
    domain::{
        AppSnapshot, Checklist, DEFAULT_CHECKLIST_ID, ReminderRepeat, TodoItem, TodoPriority,
    },
    infrastructure::{
        db::{LocalTodoAttachment, StoredConflict},
        repository::SqliteRepository,
        sync::resolve_conflict,
    },
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
fn recreating_a_record_cancels_its_unsent_local_tombstone() {
    tauri::async_runtime::block_on(async {
        let database_path = std::env::temp_dir().join(format!(
            "pixeldone-tombstone-recreate-{}.sqlite3",
            std::process::id()
        ));
        let repository = SqliteRepository::open(&database_path).await.unwrap();
        let mut before = AppSnapshot::initial(10);
        before.checklists.insert(
            1,
            Checklist::new_normal("deleted-list".to_owned(), "Deleted".to_owned(), 10),
        );
        repository.save_snapshot(&before).await.unwrap();

        let mut deleted = before.clone();
        deleted.checklists.retain(|list| list.id != "deleted-list");
        repository
            .save_snapshot_with_changes(&before, &deleted)
            .await
            .unwrap();
        assert!(
            repository
                .tombstones()
                .await
                .unwrap()
                .iter()
                .any(|value| value.record_type == "checklist" && value.local_id == "deleted-list")
        );

        let mut recreated = deleted.clone();
        recreated.checklists.insert(
            1,
            Checklist::new_normal("deleted-list".to_owned(), "Deleted".to_owned(), 11),
        );
        repository
            .save_snapshot_with_changes(&deleted, &recreated)
            .await
            .unwrap();

        assert!(
            !repository
                .tombstones()
                .await
                .unwrap()
                .iter()
                .any(|value| value.local_id == "deleted-list")
        );
        assert!(
            repository
                .dirty_records()
                .await
                .unwrap()
                .iter()
                .any(|value| value.record_type == "checklist" && value.local_id == "deleted-list")
        );

        repository.close().await;
        let _ = std::fs::remove_file(database_path);
    });
}

#[test]
fn keeping_a_locally_restored_checklist_rekeys_it_and_clears_the_conflict() {
    tauri::async_runtime::block_on(async {
        let database_path = std::env::temp_dir().join(format!(
            "pixeldone-restored-checklist-conflict-{}.sqlite3",
            std::process::id()
        ));
        let repository = SqliteRepository::open(&database_path).await.unwrap();
        let deleted_id = "deleted-list";
        let mut snapshot = AppSnapshot::initial(10);
        snapshot.checklists.insert(
            1,
            Checklist::new_normal(deleted_id.to_owned(), "Restored".to_owned(), 10),
        );
        repository.save_snapshot(&snapshot).await.unwrap();
        repository
            .mark_dirty("checklist", deleted_id)
            .await
            .unwrap();
        repository
            .save_conflict(&StoredConflict {
                record_type: "checklist".to_owned(),
                local_id: deleted_id.to_owned(),
                local_payload_json: serde_json::json!({
                    "owner_user_id": "owner",
                    "local_id": deleted_id,
                    "name": "Restored",
                    "remote_version": null
                })
                .to_string(),
                remote_payload_json: "null".to_owned(),
                fields_json: r#"["name"]"#.to_owned(),
                message: "remote_version_changed".to_owned(),
                remote_version: Some(12),
            })
            .await
            .unwrap();

        resolve_conflict(&repository, &mut snapshot, "checklist", deleted_id, false)
            .await
            .unwrap();

        let restored = snapshot
            .checklists
            .iter()
            .find(|list| list.name == "Restored")
            .unwrap();
        assert_ne!(restored.id, deleted_id);
        assert_eq!(restored.remote_version, None);
        assert!(repository.conflicts().await.unwrap().is_empty());
        let dirty = repository.dirty_records().await.unwrap();
        assert!(!dirty.iter().any(|value| value.local_id == deleted_id));
        assert!(
            dirty
                .iter()
                .any(|value| { value.record_type == "checklist" && value.local_id == restored.id })
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
        assert!((200..=720).contains(&snapshot.settings.sidebar_width_px));
        repository.close().await;
    });
}
