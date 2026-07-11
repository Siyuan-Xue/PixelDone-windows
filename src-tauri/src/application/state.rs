use std::{collections::HashMap, path::PathBuf};

use tokio::sync::{Mutex, Notify};

use crate::{
    domain::AppSnapshot,
    infrastructure::{
        auth::{AuthSession, SupabaseClient},
        repository::SqliteRepository,
    },
    platform::windows::credentials::CredentialStore,
};

pub struct RuntimeState {
    pub snapshot: AppSnapshot,
    pub repository: SqliteRepository,
}

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub root: PathBuf,
    pub data: PathBuf,
    pub attachments: PathBuf,
    pub cache: PathBuf,
    pub logs: PathBuf,
}

pub struct ManagedAppState {
    pub inner: Mutex<RuntimeState>,
    pub cloud: SupabaseClient,
    pub credentials: CredentialStore,
    pub session: Mutex<Option<AuthSession>>,
    pub sync_gate: Mutex<()>,
    pub sync_notify: Notify,
    pub fired_reminders: Mutex<HashMap<String, i64>>,
    pub snoozed_until: Mutex<HashMap<String, i64>>,
    pub paths: AppPaths,
}

impl ManagedAppState {
    pub fn new(
        snapshot: AppSnapshot,
        repository: SqliteRepository,
        cloud: SupabaseClient,
        credentials: CredentialStore,
        session: Option<AuthSession>,
        paths: AppPaths,
    ) -> Self {
        Self {
            inner: Mutex::new(RuntimeState {
                snapshot,
                repository,
            }),
            cloud,
            credentials,
            session: Mutex::new(session),
            sync_gate: Mutex::new(()),
            sync_notify: Notify::new(),
            fired_reminders: Mutex::new(HashMap::new()),
            snoozed_until: Mutex::new(HashMap::new()),
            paths,
        }
    }
}
