use serde::ser::{Serialize, SerializeStruct, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("当前界面状态已过期，请使用最新数据重试")]
    StaleRevision,
    #[error("{0}")]
    Validation(String),
    #[error("未找到请求的数据：{0}")]
    NotFound(String),
    #[error("数据库操作失败：{0}")]
    Database(String),
    #[error("网络请求失败：{0}")]
    Network(String),
    #[error("账号操作失败：{0}")]
    Auth(String),
    #[error("SERVER UPDATE REQUIRED: {0}")]
    ServerUpdateRequired(String),
    #[error("Remote sync data is invalid: {0}")]
    RemoteDataInvalid(String),
    #[error("Windows 系统操作失败：{0}")]
    Platform(String),
    #[error("Windows 通知已禁用：{0}")]
    NotificationsDisabled(String),
    #[error("更新操作失败：{0}")]
    Update(String),
    #[error("应用初始化失败：{0}")]
    Initialization(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::StaleRevision => "STALE_REVISION",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Network(_) => "NETWORK_ERROR",
            Self::Auth(_) => "AUTH_ERROR",
            Self::ServerUpdateRequired(_) => "SERVER_UPDATE_REQUIRED",
            Self::RemoteDataInvalid(_) => "REMOTE_DATA_INVALID",
            Self::Platform(_) => "PLATFORM_ERROR",
            Self::NotificationsDisabled(_) => "NOTIFICATIONS_DISABLED",
            Self::Update(_) => "UPDATE_ERROR",
            Self::Initialization(_) => "INITIALIZATION_ERROR",
        }
    }

    pub fn sync_issue_code(&self) -> crate::domain::SyncIssueCode {
        use crate::domain::SyncIssueCode;
        match self {
            Self::Network(_) => SyncIssueCode::NetworkRetrying,
            Self::Auth(_) => SyncIssueCode::AuthExpired,
            Self::ServerUpdateRequired(_) => SyncIssueCode::ServerUpdateRequired,
            Self::Database(_) => SyncIssueCode::LocalStorageError,
            Self::RemoteDataInvalid(_) => SyncIssueCode::RemoteDataInvalid,
            _ => SyncIssueCode::Unknown,
        }
    }

    pub fn is_retryable_sync_error(&self) -> bool {
        matches!(self, Self::Network(_))
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self {
        Self::Database(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Initialization(value.to_string())
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Network(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::SyncIssueCode;

    #[test]
    fn sync_errors_have_stable_public_categories() {
        assert_eq!(
            AppError::Network("offline".into()).sync_issue_code(),
            SyncIssueCode::NetworkRetrying
        );
        assert!(AppError::Network("offline".into()).is_retryable_sync_error());
        assert_eq!(
            AppError::Auth("expired".into()).sync_issue_code(),
            SyncIssueCode::AuthExpired
        );
        assert!(!AppError::Auth("expired".into()).is_retryable_sync_error());
        assert_eq!(
            AppError::ServerUpdateRequired("schema".into()).code(),
            "SERVER_UPDATE_REQUIRED"
        );
        assert_eq!(
            AppError::RemoteDataInvalid("payload".into()).sync_issue_code(),
            SyncIssueCode::RemoteDataInvalid
        );
    }
}
