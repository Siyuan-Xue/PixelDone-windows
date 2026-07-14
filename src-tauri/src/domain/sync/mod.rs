use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SyncState {
    LocalOnly,
    SignedOut,
    Idle,
    Syncing,
    #[default]
    Synced,
    Conflict,
    Error,
    ServerUpdateRequired,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunView {
    pub state: SyncState,
    pub message: Option<String>,
    pub remote_version: Option<i64>,
    pub pending_count: usize,
    pub conflict_count: usize,
    pub insecure_http: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthView {
    pub cloud_available: bool,
    pub signed_in: bool,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub insecure_http: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflictView {
    pub record_type: String,
    pub local_id: String,
    pub title: String,
    pub fields: Vec<SyncConflictFieldView>,
    pub message_code: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflictFieldView {
    pub key: String,
    pub local_value: SyncConflictValueView,
    pub cloud_value: SyncConflictValueView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncConflictValueView {
    pub kind: String,
    pub value: serde_json::Value,
    pub label: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictResolutionChoice {
    KeepLocal,
    KeepCloud,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AppLanguage {
    #[default]
    System,
    English,
    SimplifiedChinese,
    Arabic,
    French,
    Russian,
    Spanish,
}

impl AppLanguage {
    pub fn sync_value(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::English => "en",
            Self::SimplifiedChinese => "zh-Hans",
            Self::Arabic => "ar",
            Self::French => "fr",
            Self::Russian => "ru",
            Self::Spanish => "es",
        }
    }

    pub fn from_sync_value(value: &str) -> Self {
        match value {
            "en" => Self::English,
            "zh-Hans" => Self::SimplifiedChinese,
            "ar" => Self::Arabic,
            "fr" => Self::French,
            "ru" => Self::Russian,
            "es" => Self::Spanish,
            _ => Self::System,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateView {
    pub state: String,
    pub current_version: String,
    pub available_version: Option<String>,
    pub download_url: Option<String>,
    pub source: Option<String>,
    pub message: Option<String>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub last_checked_at_millis: Option<i64>,
    pub next_check_at_millis: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderRunView {
    pub state: String,
    pub active_todo_ids: Vec<String>,
    pub last_fired_at_millis: Option<i64>,
    pub scheduled_count: usize,
    pub schedule_horizon_at_millis: Option<i64>,
    pub schedule_truncated: bool,
    pub message: Option<String>,
}
