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
            .timeout(std::time::Duration::from_secs(20))
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

    pub async fn reset_password(&self, email: &str) -> Result<(), AppError> {
        self.request::<Value>(
            Method::POST,
            "/auth/v1/recover",
            None,
            Some(json!({ "email": email.trim() })),
        )
        .await?;
        Ok(())
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
        serde_json::from_str(&text).map_err(|error| AppError::Network(error.to_string()))
    }
}
