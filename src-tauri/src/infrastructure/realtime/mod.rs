use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::Notify;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    domain::{AppError, unix_now_millis},
    infrastructure::auth::{AuthSession, SupabaseClient},
};

const TABLES: [&str; 4] = [
    "todo_checklists",
    "todo_items",
    "user_settings",
    "sync_tombstones",
];
const HEARTBEAT_SECONDS: u64 = 25;
const REFRESH_SKEW_MILLIS: i64 = 60_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionExit {
    AuthenticationChanged,
    TokenRefreshRequired,
}

pub async fn listen_for_invalidations(
    cloud: &SupabaseClient,
    session: &AuthSession,
    sync_notify: &Notify,
    auth_notify: &Notify,
) -> Result<ConnectionExit, AppError> {
    let realtime_url = cloud.realtime_url()?;
    let (stream, _) = tokio::time::timeout(
        Duration::from_secs(10),
        connect_async(realtime_url.as_str()),
    )
    .await
    .map_err(|_| AppError::Network("Supabase Realtime connection timed out".to_owned()))?
    .map_err(network_error)?;
    let (mut writer, mut reader) = stream.split();
    let topic = format!("realtime:pixeldone-{}", session.user_id);
    writer
        .send(Message::Text(
            join_payload(&topic, session).to_string().into(),
        ))
        .await
        .map_err(network_error)?;

    let mut heartbeat = tokio::time::interval(Duration::from_secs(HEARTBEAT_SECONDS));
    heartbeat.tick().await;
    let refresh_delay = refresh_delay(session.expires_at_millis, unix_now_millis());
    let refresh = tokio::time::sleep(refresh_delay);
    tokio::pin!(refresh);
    let mut reference = 2_u64;

    loop {
        tokio::select! {
            () = auth_notify.notified() => return Ok(ConnectionExit::AuthenticationChanged),
            () = &mut refresh => return Ok(ConnectionExit::TokenRefreshRequired),
            _ = heartbeat.tick() => {
                writer
                    .send(Message::Text(heartbeat_payload(reference).to_string().into()))
                    .await
                    .map_err(network_error)?;
                reference += 1;
            }
            message = reader.next() => {
                let message = message
                    .ok_or_else(|| AppError::Network("Supabase Realtime connection closed".to_owned()))?
                    .map_err(network_error)?;
                match message {
                    Message::Text(text) => {
                        let value: Value = serde_json::from_str(text.as_ref())
                            .map_err(|error| AppError::Network(format!("Invalid Realtime payload: {error}")))?;
                        if is_join_success(&value) || is_postgres_invalidation(&value) {
                            sync_notify.notify_one();
                        }
                        if is_channel_error(&value) {
                            return Err(AppError::Network("Supabase Realtime channel error".to_owned()));
                        }
                    }
                    Message::Ping(value) => writer.send(Message::Pong(value)).await.map_err(network_error)?,
                    Message::Close(_) => return Err(AppError::Network("Supabase Realtime connection closed".to_owned())),
                    _ => {}
                }
            }
        }
    }
}

pub fn realtime_websocket_url(base_url: &str, publishable_key: &str) -> Result<url::Url, AppError> {
    let mut url = url::Url::parse(base_url)
        .map_err(|error| AppError::Network(format!("Invalid Supabase URL: {error}")))?;
    let scheme = match url.scheme() {
        "http" => "ws",
        "https" => "wss",
        value => {
            return Err(AppError::Network(format!(
                "Unsupported Supabase URL scheme: {value}"
            )));
        }
    };
    url.set_scheme(scheme)
        .map_err(|_| AppError::Network("Could not derive Realtime URL".to_owned()))?;
    url.set_path("/realtime/v1/websocket");
    url.set_query(None);
    url.query_pairs_mut()
        .append_pair("apikey", publishable_key)
        .append_pair("vsn", "1.0.0");
    Ok(url)
}

pub fn retry_delay(attempt: u32) -> Duration {
    Duration::from_secs(2_u64.saturating_pow(attempt.min(5)).min(30))
}

fn join_payload(topic: &str, session: &AuthSession) -> Value {
    let filter = format!("owner_user_id=eq.{}", session.user_id);
    let postgres_changes = TABLES
        .iter()
        .map(|table| {
            json!({
                "event": "*",
                "schema": "public",
                "table": table,
                "filter": filter,
            })
        })
        .collect::<Vec<_>>();
    json!({
        "topic": topic,
        "event": "phx_join",
        "payload": {
            "config": {
                "broadcast": { "ack": false, "self": false },
                "presence": { "enabled": false },
                "postgres_changes": postgres_changes,
            },
            "access_token": session.access_token,
        },
        "ref": "1",
        "join_ref": "1",
    })
}

fn heartbeat_payload(reference: u64) -> Value {
    json!({
        "topic": "phoenix",
        "event": "heartbeat",
        "payload": {},
        "ref": reference.to_string(),
        "join_ref": null,
    })
}

fn refresh_delay(expires_at_millis: i64, now_millis: i64) -> Duration {
    Duration::from_millis((expires_at_millis - REFRESH_SKEW_MILLIS - now_millis).max(5_000) as u64)
}

fn is_join_success(value: &Value) -> bool {
    value.get("event").and_then(Value::as_str) == Some("phx_reply")
        && value
            .get("topic")
            .and_then(Value::as_str)
            .is_some_and(|topic| topic.starts_with("realtime:"))
        && value.get("ref").and_then(Value::as_str) == Some("1")
        && value.pointer("/payload/status").and_then(Value::as_str) == Some("ok")
}

fn is_postgres_invalidation(value: &Value) -> bool {
    value.get("event").and_then(Value::as_str) == Some("postgres_changes")
}

fn is_channel_error(value: &Value) -> bool {
    matches!(
        value.get("event").and_then(Value::as_str),
        Some("phx_error" | "phx_close" | "system")
    ) || (value.get("event").and_then(Value::as_str) == Some("phx_reply")
        && value.pointer("/payload/status").and_then(Value::as_str) == Some("error"))
}

fn network_error(error: tokio_tungstenite::tungstenite::Error) -> AppError {
    AppError::Network(format!("Supabase Realtime: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> AuthSession {
        AuthSession {
            user_id: "user-1".into(),
            email: None,
            access_token: "access-token".into(),
            refresh_token: "refresh-token".into(),
            expires_at_millis: 100_000,
        }
    }

    #[test]
    fn realtime_url_uses_websocket_protocol_and_official_path() {
        let url = realtime_websocket_url("http://localhost:8000", "key").unwrap();
        assert_eq!(url.scheme(), "ws");
        assert_eq!(url.path(), "/realtime/v1/websocket");
        assert!(url.query().unwrap().contains("apikey=key"));
        assert!(url.query().unwrap().contains("vsn=1.0.0"));
    }

    #[test]
    fn join_subscribes_to_all_change_tables_for_the_signed_in_user() {
        let payload = join_payload("realtime:pixeldone-user-1", &session());
        let changes = payload
            .pointer("/payload/config/postgres_changes")
            .unwrap()
            .as_array()
            .unwrap();
        assert_eq!(changes.len(), 4);
        for table in TABLES {
            assert!(changes.iter().any(|value| value["table"] == table));
        }
        assert!(
            changes
                .iter()
                .all(|value| value["filter"] == "owner_user_id=eq.user-1")
        );
    }

    #[test]
    fn protocol_events_are_classified_without_applying_payloads() {
        assert!(is_join_success(
            &json!({"topic":"realtime:pixeldone-user-1","event":"phx_reply","ref":"1","payload":{"status":"ok"}})
        ));
        assert!(!is_join_success(
            &json!({"topic":"phoenix","event":"phx_reply","ref":"9","payload":{"status":"ok"}})
        ));
        assert!(is_postgres_invalidation(
            &json!({"event":"postgres_changes"})
        ));
        assert!(is_channel_error(&json!({"event":"phx_error"})));
        assert_eq!(retry_delay(0), Duration::from_secs(1));
        assert_eq!(retry_delay(8), Duration::from_secs(30));
    }

    #[test]
    fn token_refresh_is_scheduled_before_expiry() {
        assert_eq!(refresh_delay(100_000, 10_000), Duration::from_secs(30));
        assert_eq!(refresh_delay(50_000, 49_000), Duration::from_secs(5));
        let heartbeat = heartbeat_payload(9);
        assert_eq!(heartbeat["event"], "heartbeat");
        assert_eq!(heartbeat["ref"], "9");
    }
}
