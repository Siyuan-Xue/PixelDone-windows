use pixeldone_windows_lib::{
    domain::AppSnapshot,
    infrastructure::{db::LocalTodoAttachment, repository::SqliteRepository},
};

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
        assert_eq!(snapshot.settings.sidebar_width_px, 256);
        snapshot.settings.enhanced_xhigh_alarm_enabled = true;
        snapshot.settings.sidebar_width_px = 344;
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
