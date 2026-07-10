use tokio::sync::Mutex;

use crate::{domain::AppSnapshot, infrastructure::repository::SqliteRepository};

pub struct RuntimeState {
    pub snapshot: AppSnapshot,
    pub repository: SqliteRepository,
}

pub struct ManagedAppState {
    pub inner: Mutex<RuntimeState>,
}

impl ManagedAppState {
    pub fn new(snapshot: AppSnapshot, repository: SqliteRepository) -> Self {
        Self {
            inner: Mutex::new(RuntimeState {
                snapshot,
                repository,
            }),
        }
    }
}
