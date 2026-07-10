use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncState {
    #[default]
    LocalOnly,
    Pending,
    Synced,
    Conflict,
    Error,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunView {
    pub state: SyncState,
    pub message: Option<String>,
    pub remote_version: Option<i64>,
}
