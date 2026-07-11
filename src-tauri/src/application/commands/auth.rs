use tauri::State;

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, SyncRunView, SyncState},
};

#[tauri::command]
pub async fn auth_sign_in(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    email: String,
    password: String,
) -> Result<MutationResult, AppError> {
    validate_credentials(&email, &password)?;
    let session = state.cloud.sign_in(&email, &password).await?;
    state.credentials.save(&session)?;
    *state.session.lock().await = Some(session.clone());
    let auth = state.cloud.auth_view(Some(&session));
    mutate(state, expected_revision, move |snapshot| {
        snapshot.auth = auth;
        snapshot.sync = SyncRunView {
            state: SyncState::Idle,
            message: Some("已登录，正在等待同步".to_owned()),
            insecure_http: true,
            ..SyncRunView::default()
        };
        Ok(vec!["auth".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn auth_sign_up(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    email: String,
    password: String,
) -> Result<MutationResult, AppError> {
    validate_credentials(&email, &password)?;
    let session = state.cloud.sign_up(&email, &password).await?;
    state.credentials.save(&session)?;
    *state.session.lock().await = Some(session.clone());
    let auth = state.cloud.auth_view(Some(&session));
    mutate(state, expected_revision, move |snapshot| {
        snapshot.auth = auth;
        snapshot.sync = SyncRunView {
            state: SyncState::Idle,
            message: Some("账号已创建，正在等待同步".to_owned()),
            insecure_http: true,
            ..SyncRunView::default()
        };
        Ok(vec!["auth".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn auth_sign_out(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    if let Some(session) = state.session.lock().await.take() {
        state.cloud.sign_out(&session).await?;
    }
    state.credentials.clear()?;
    let auth = state.cloud.auth_view(None);
    mutate(state, expected_revision, move |snapshot| {
        snapshot.auth = auth;
        snapshot.sync = SyncRunView {
            state: SyncState::SignedOut,
            message: Some("已退出账号".to_owned()),
            insecure_http: true,
            ..SyncRunView::default()
        };
        Ok(vec!["auth".to_owned()])
    })
    .await
}

#[tauri::command]
pub async fn auth_reset_password(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    email: String,
) -> Result<MutationResult, AppError> {
    if email.trim().is_empty() {
        return Err(AppError::Validation("请输入邮箱".to_owned()));
    }
    state.cloud.reset_password(&email).await?;
    mutate(state, expected_revision, |snapshot| {
        snapshot.sync.message = Some("密码重置邮件已发送".to_owned());
        Ok(vec!["auth".to_owned()])
    })
    .await
}

fn validate_credentials(email: &str, password: &str) -> Result<(), AppError> {
    if email.trim().is_empty() || password.is_empty() {
        return Err(AppError::Validation("邮箱和密码不能为空".to_owned()));
    }
    if password.len() < 6 {
        return Err(AppError::Validation("密码至少需要 6 个字符".to_owned()));
    }
    Ok(())
}
