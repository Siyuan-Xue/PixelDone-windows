use std::path::PathBuf;

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
    pub webview_data: PathBuf,
    pub legacy_roaming_database: PathBuf,
}

pub struct ManagedAppState {
    pub inner: Mutex<RuntimeState>,
    pub cloud: SupabaseClient,
    pub credentials: CredentialStore,
    pub session: Mutex<Option<AuthSession>>,
    pub sync_gate: Mutex<()>,
    pub sync_notify: Notify,
    pub reminder_gate: Mutex<()>,
    pub reminder_notify: Notify,
    pub auth_notify: Notify,
    pub paths: AppPaths,
    pub notification_identity_error: Option<String>,
}

impl ManagedAppState {
    pub fn new(
        snapshot: AppSnapshot,
        repository: SqliteRepository,
        cloud: SupabaseClient,
        credentials: CredentialStore,
        session: Option<AuthSession>,
        paths: AppPaths,
        notification_identity_error: Option<String>,
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
            reminder_gate: Mutex::new(()),
            reminder_notify: Notify::new(),
            auth_notify: Notify::new(),
            paths,
            notification_identity_error,
        }
    }
}
