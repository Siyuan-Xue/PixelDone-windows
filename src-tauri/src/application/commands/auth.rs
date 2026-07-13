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
    state.auth_notify.notify_one();
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
    state.auth_notify.notify_one();
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
    let session = state.session.lock().await.take();
    state.auth_notify.notify_one();
    if let Some(session) = session {
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
pub async fn auth_change_password(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    current_password: String,
    new_password: String,
    confirmation: String,
) -> Result<MutationResult, AppError> {
    if current_password.is_empty() || new_password.is_empty() || confirmation.is_empty() {
        return Err(AppError::Validation(
            "All password fields are required".to_owned(),
        ));
    }
    if new_password != confirmation {
        return Err(AppError::Validation(
            "New passwords do not match".to_owned(),
        ));
    }
    if current_password == new_password {
        return Err(AppError::Validation(
            "Choose a different new password".to_owned(),
        ));
    }
    if new_password.len() < 6 {
        return Err(AppError::Validation(
            "New password must contain at least 6 characters".to_owned(),
        ));
    }
    let session = state
        .session
        .lock()
        .await
        .clone()
        .ok_or_else(|| AppError::Auth("Sign in first".to_owned()))?;
    let global_logout = state
        .cloud
        .change_password(&session, &current_password, &new_password)
        .await?;
    state.credentials.clear()?;
    *state.session.lock().await = None;
    state.auth_notify.notify_one();
    let auth = state.cloud.auth_view(None);
    mutate(state, expected_revision, move |snapshot| {
        snapshot.auth = auth;
        snapshot.sync = SyncRunView {
            state: SyncState::SignedOut,
            message: Some(if global_logout {
                "Password changed. Sign in again".to_owned()
            } else {
                "Password changed. This device signed out; some other sessions may still be active"
                    .to_owned()
            }),
            insecure_http: true,
            ..SyncRunView::default()
        };
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
