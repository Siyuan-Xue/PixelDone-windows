use pixeldone_windows_lib::{domain::AppSnapshot, infrastructure::repository::SqliteRepository};

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
        let snapshot = AppSnapshot::initial(42);
        repository
            .save_snapshot(&snapshot)
            .await
            .expect("snapshot should persist");
        let restored = repository
            .load_snapshot()
            .await
            .expect("snapshot should load")
            .expect("metadata row should exist");

        assert_eq!(restored, snapshot);
        repository.close().await;
        let _ = std::fs::remove_file(database_path);
    });
}
