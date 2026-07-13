use reqwest::{Client, Method, StatusCode};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};

use crate::domain::{AppError, AuthView, unix_now_millis};

mod build_config {
    include!(concat!(env!("OUT_DIR"), "/pixeldone_cloud_config.rs"));
}

const REFRESH_SKEW_MILLIS: i64 = 60_000;

#[derive(Clone)]
pub struct SupabaseClient {
    client: Client,
    pub base_url: String,
    publishable_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSession {
    pub user_id: String,
    pub email: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_millis: i64,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    user: TokenUser,
}

#[derive(Deserialize)]
struct TokenUser {
    id: String,
    email: Option<String>,
}

impl SupabaseClient {
    pub(crate) fn realtime_url(&self) -> Result<url::Url, AppError> {
        crate::infrastructure::realtime::realtime_websocket_url(
            &self.base_url,
            &self.publishable_key,
        )
    }

    pub fn from_build_config() -> Result<Self, AppError> {
        let base_url = build_config::SUPABASE_URL.trim_end_matches('/').to_owned();
        let publishable_key = build_config::SUPABASE_PUBLISHABLE_KEY.to_owned();
        if base_url.is_empty() || publishable_key.is_empty() {
            return Err(AppError::Initialization("Supabase 构建配置缺失".to_owned()));
        }
        if !base_url.starts_with("http://") {
            return Err(AppError::Initialization(
                "PixelDone 3.1 仅使用已批准的 HTTP Supabase 地址".to_owned(),
            ));
        }
        if !build_config::ALLOW_INSECURE_HTTP {
            return Err(AppError::Initialization("HTTP 云端配置未获允许".to_owned()));
        }
        let client = Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(60))
            .build()?;
        Ok(Self {
            client,
            base_url,
            publishable_key,
        })
    }

    pub fn auth_view(&self, session: Option<&AuthSession>) -> AuthView {
        AuthView {
            cloud_available: true,
            signed_in: session.is_some(),
            user_id: session.map(|value| value.user_id.clone()),
            user_email: session.and_then(|value| value.email.clone()),
            insecure_http: true,
        }
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> Result<AuthSession, AppError> {
        self.token_request(
            "/auth/v1/token?grant_type=password",
            json!({ "email": email.trim(), "password": password }),
            None,
        )
        .await
    }

    pub async fn sign_up(&self, email: &str, password: &str) -> Result<AuthSession, AppError> {
        self.token_request(
            "/auth/v1/signup",
            json!({ "email": email.trim(), "password": password }),
            None,
        )
        .await
    }

    pub async fn change_password(
        &self,
        session: &AuthSession,
        current_password: &str,
        new_password: &str,
    ) -> Result<bool, AppError> {
        let email = session.email.as_deref().ok_or_else(|| {
            AppError::Auth("The signed-in account has no email address".to_owned())
        })?;
        let reauthenticated = self.sign_in(email, current_password).await?;
        if reauthenticated.user_id != session.user_id {
            return Err(AppError::Auth(
                "Password verification returned a different account".to_owned(),
            ));
        }
        self.request::<Value>(
            Method::PUT,
            "/auth/v1/user",
            Some(&reauthenticated.access_token),
            Some(json!({ "password": new_password })),
        )
        .await?;
        Ok(self
            .request::<Value>(
                Method::POST,
                "/auth/v1/logout?scope=global",
                Some(&reauthenticated.access_token),
                None,
            )
            .await
            .is_ok())
    }

    pub async fn sign_out(&self, session: &AuthSession) -> Result<(), AppError> {
        let _ = self
            .request::<Value>(
                Method::POST,
                "/auth/v1/logout",
                Some(&session.access_token),
                None,
            )
            .await;
        Ok(())
    }

    pub async fn refresh_if_needed(
        &self,
        session: &AuthSession,
        force: bool,
    ) -> Result<AuthSession, AppError> {
        if !force && session.expires_at_millis - REFRESH_SKEW_MILLIS > unix_now_millis() {
            return Ok(session.clone());
        }
        self.token_request(
            "/auth/v1/token?grant_type=refresh_token",
            json!({ "refresh_token": session.refresh_token }),
            Some(session),
        )
        .await
    }

    pub async fn rpc<T: DeserializeOwned>(
        &self,
        session: &AuthSession,
        function: &str,
        body: Value,
    ) -> Result<T, AppError> {
        self.request(
            Method::POST,
            &format!("/rest/v1/rpc/{function}"),
            Some(&session.access_token),
            Some(body),
        )
        .await
    }

    pub async fn upload_todo_image(
        &self,
        session: &AuthSession,
        object_path: &str,
        content_type: &str,
        bytes: Vec<u8>,
    ) -> Result<(), AppError> {
        let response = self
            .client
            .post(format!(
                "{}/storage/v1/object/pixeldone-todo-images/{object_path}",
                self.base_url
            ))
            .header("apikey", &self.publishable_key)
            .bearer_auth(&session.access_token)
            .header("Content-Type", content_type)
            .header("x-upsert", "true")
            .body(bytes)
            .send()
            .await?;
        require_storage_success(response).await.map(|_| ())
    }

    pub async fn download_todo_image(
        &self,
        session: &AuthSession,
        object_path: &str,
    ) -> Result<Vec<u8>, AppError> {
        let response = self
            .client
            .get(format!(
                "{}/storage/v1/object/authenticated/pixeldone-todo-images/{object_path}",
                self.base_url
            ))
            .header("apikey", &self.publishable_key)
            .bearer_auth(&session.access_token)
            .send()
            .await?;
        require_storage_success(response).await
    }

    pub async fn delete_todo_image_object(
        &self,
        session: &AuthSession,
        object_path: &str,
    ) -> Result<(), AppError> {
        let response = self
            .client
            .delete(format!(
                "{}/storage/v1/object/pixeldone-todo-images",
                self.base_url
            ))
            .header("apikey", &self.publishable_key)
            .bearer_auth(&session.access_token)
            .json(&json!({ "prefixes": [object_path] }))
            .send()
            .await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        require_storage_success(response).await.map(|_| ())
    }

    async fn token_request(
        &self,
        path: &str,
        body: Value,
        previous: Option<&AuthSession>,
    ) -> Result<AuthSession, AppError> {
        let response: TokenResponse = self.request(Method::POST, path, None, Some(body)).await?;
        let refresh_token = response
            .refresh_token
            .or_else(|| previous.map(|value| value.refresh_token.clone()))
            .ok_or_else(|| AppError::Auth("Supabase 未返回 refresh token".to_owned()))?;
        Ok(AuthSession {
            user_id: response.user.id,
            email: response
                .user
                .email
                .or_else(|| previous.and_then(|value| value.email.clone())),
            access_token: response.access_token,
            refresh_token,
            expires_at_millis: unix_now_millis() + response.expires_in.unwrap_or(3600) * 1_000,
        })
    }

    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        path: &str,
        bearer: Option<&str>,
        body: Option<Value>,
    ) -> Result<T, AppError> {
        let mut request = self
            .client
            .request(method, format!("{}{}", self.base_url, path))
            .header("Accept", "application/json")
            .header("apikey", &self.publishable_key);
        if let Some(token) = bearer {
            request = request.bearer_auth(token);
        }
        if let Some(body) = body {
            request = request.json(&body);
        }
        let response = request.send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            let message = serde_json::from_str::<Value>(&text)
                .ok()
                .and_then(|value| {
                    value
                        .get("msg")
                        .or_else(|| value.get("message"))
                        .or_else(|| value.get("error_description"))
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
                .unwrap_or_else(|| format!("HTTP {status}: {text}"));
            return if status == StatusCode::UNAUTHORIZED || path.starts_with("/auth/") {
                Err(AppError::Auth(message))
            } else {
                Err(AppError::Network(message))
            };
        }
        serde_json::from_str(if text.trim().is_empty() {
            "null"
        } else {
            &text
        })
        .map_err(|error| AppError::Network(error.to_string()))
    }
}

async fn require_storage_success(response: reqwest::Response) -> Result<Vec<u8>, AppError> {
    let status = response.status();
    let bytes = response.bytes().await?;
    if status.is_success() {
        return Ok(bytes.to_vec());
    }
    let text = String::from_utf8_lossy(&bytes);
    let message = serde_json::from_slice::<Value>(&bytes)
        .ok()
        .and_then(|value| {
            value
                .get("message")
                .or_else(|| value.get("error"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("Supabase Storage HTTP {status}: {text}"));
    Err(AppError::Network(message))
}
