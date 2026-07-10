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
            Self::Initialization(_) => "INITIALIZATION_ERROR",
        }
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
