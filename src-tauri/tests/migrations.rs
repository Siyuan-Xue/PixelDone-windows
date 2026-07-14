use pixeldone_windows_lib::{
    domain::AppSnapshot,
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
