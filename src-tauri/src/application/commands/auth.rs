use tauri::State;

use crate::{
    application::{commands::ensure_revision, state::ManagedAppState},
    domain::{
        AppError, AppSnapshot, AuthView, MutationResult, SnapshotDelta, SyncRunView, SyncState,
    },
};

#[tauri::command]
pub async fn auth_sign_in(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    email: String,
    password: String,
) -> Result<MutationResult, AppError> {
    validate_credentials(&email, &password)?;
    ensure_revision(&state, expected_revision).await?;
    let session = state.cloud.sign_in(&email, &password).await?;
    if let Err(error) = state.credentials.save(&session) {
        let _ = state.cloud.sign_out(&session).await;
        return Err(error);
    }
    *state.session.lock().await = Some(session.clone());
    state.auth_notify.notify_one();
    let auth = state.cloud.auth_view(Some(&session));
    converge_auth_state(state, move |snapshot| {
        apply_signed_in(snapshot, auth, "Signed in. Waiting to sync");
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
    ensure_revision(&state, expected_revision).await?;
    let session = state.cloud.sign_up(&email, &password).await?;
    if let Err(error) = state.credentials.save(&session) {
        let _ = state.cloud.sign_out(&session).await;
        return Err(error);
    }
    *state.session.lock().await = Some(session.clone());
    state.auth_notify.notify_one();
    let auth = state.cloud.auth_view(Some(&session));
    converge_auth_state(state, move |snapshot| {
        apply_signed_in(snapshot, auth, "Account created. Waiting to sync");
    })
    .await
}

#[tauri::command]
pub async fn auth_sign_out(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
) -> Result<MutationResult, AppError> {
    ensure_revision(&state, expected_revision).await?;
    let session = state.session.lock().await.clone();
    if let Some(session) = &session {
        state.cloud.sign_out(session).await?;
    }
    state.credentials.clear()?;
    *state.session.lock().await = None;
    state.auth_notify.notify_one();
    let auth = state.cloud.auth_view(None);
    converge_auth_state(state, move |snapshot| {
        apply_signed_out(snapshot, auth, "Signed out");
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
    validate_password_change(&current_password, &new_password, &confirmation)?;
    ensure_revision(&state, expected_revision).await?;
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
    converge_auth_state(state, move |snapshot| {
        apply_signed_out(
            snapshot,
            auth,
            if global_logout {
                "Password changed. Sign in again"
            } else {
                "Password changed. This device signed out; some other sessions may still be active"
            },
        );
    })
    .await
}

async fn converge_auth_state<F>(
    state: State<'_, ManagedAppState>,
    operation: F,
) -> Result<MutationResult, AppError>
where
    F: FnOnce(&mut AppSnapshot),
{
    let mut runtime = state.inner.lock().await;
    let before = runtime.snapshot.clone();
    let mut candidate = before.clone();
    operation(&mut candidate);
    candidate.revision += 1;
    let save_result = runtime.repository.save_snapshot(&candidate).await;
    let snapshot_delta = SnapshotDelta::between(&before, &candidate);
    runtime.snapshot = candidate;
    save_result?;
    state.sync_notify.notify_one();
    state.reminder_notify.notify_one();
    Ok(MutationResult {
        revision: runtime.snapshot.revision,
        changed_ids: vec!["auth".to_owned()],
        snapshot_delta,
    })
}

fn apply_signed_in(snapshot: &mut AppSnapshot, auth: AuthView, message: &str) {
    snapshot.auth = auth;
    snapshot.sync = SyncRunView {
        state: SyncState::Idle,
        message: Some(message.to_owned()),
        insecure_http: true,
        ..SyncRunView::default()
    };
}

fn apply_signed_out(snapshot: &mut AppSnapshot, auth: AuthView, message: &str) {
    snapshot.auth = auth;
    snapshot.sync = SyncRunView {
        state: SyncState::SignedOut,
        message: Some(message.to_owned()),
        insecure_http: true,
        ..SyncRunView::default()
    };
}

fn validate_credentials(email: &str, password: &str) -> Result<(), AppError> {
    if email.trim().is_empty() || password.is_empty() {
        return Err(AppError::Validation(
            "Email and password are required".to_owned(),
        ));
    }
    if password.chars().count() < 6 {
        return Err(AppError::Validation(
            "Password must contain at least 6 characters".to_owned(),
        ));
    }
    Ok(())
}

fn validate_password_change(
    current_password: &str,
    new_password: &str,
    confirmation: &str,
) -> Result<(), AppError> {
    if current_password.is_empty() || new_password.is_empty() || confirmation.is_empty() {
        return Err(AppError::Validation(
            "All password fields are required".to_owned(),
        ));
    }
    if new_password.chars().count() < 6 {
        return Err(AppError::Validation(
            "New password must contain at least 6 characters".to_owned(),
        ));
    }
    if current_password == new_password {
        return Err(AppError::Validation(
            "Choose a different new password".to_owned(),
        ));
    }
    if new_password != confirmation {
        return Err(AppError::Validation(
            "New passwords do not match".to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_validation_uses_unicode_characters_and_all_rules() {
        assert!(validate_password_change("", "", "").is_err());
        assert!(
            validate_password_change(
                "old",
                "\u{4e00}\u{4e8c}\u{4e09}\u{56db}\u{4e94}",
                "\u{4e00}\u{4e8c}\u{4e09}\u{56db}\u{4e94}"
            )
            .is_err()
        );
        assert!(validate_password_change("same-secret", "same-secret", "same-secret").is_err());
        assert!(validate_password_change("old-secret", "new-secret", "different").is_err());
        assert!(
            validate_password_change(
                "old-secret",
                "\u{4e00}\u{4e8c}\u{4e09}\u{56db}\u{4e94}\u{516d}",
                "\u{4e00}\u{4e8c}\u{4e09}\u{56db}\u{4e94}\u{516d}"
            )
            .is_ok()
        );
    }

    #[test]
    fn signed_out_convergence_always_clears_account_and_sync_retry_state() {
        let mut snapshot = AppSnapshot::initial(0);
        snapshot.auth.signed_in = true;
        snapshot.sync.issue_code = Some(crate::domain::SyncIssueCode::NetworkRetrying);
        snapshot.sync.next_retry_at_millis = Some(99);
        apply_signed_out(
            &mut snapshot,
            AuthView {
                cloud_available: true,
                insecure_http: true,
                ..AuthView::default()
            },
            "Password changed",
        );
        assert!(!snapshot.auth.signed_in);
        assert_eq!(snapshot.sync.state, SyncState::SignedOut);
        assert_eq!(snapshot.sync.issue_code, None);
        assert_eq!(snapshot.sync.next_retry_at_millis, None);
    }
}
