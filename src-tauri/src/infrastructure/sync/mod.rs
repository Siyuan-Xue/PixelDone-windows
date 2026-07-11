use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    domain::{
        AppError, AppLanguage, AppSnapshot, Checklist, ChecklistKind, ReminderRepeat,
        SyncConflictView, SyncRunView, SyncState, TodoItem, TodoPriority, unix_now_millis,
    },
    infrastructure::{
        auth::{AuthSession, SupabaseClient},
        db::{LocalTombstone, SqliteRepository, StoredConflict},
    },
};

const EXPECTED_SCHEMA: &str = "3.1";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct RemoteSnapshot {
    #[serde(default)]
    checklists: Vec<RemoteChecklist>,
    #[serde(default)]
    items: Vec<RemoteTodo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteChangeBatch {
    schema_version: String,
    server_version: i64,
    #[serde(default)]
    checklists: Vec<RemoteChecklist>,
    #[serde(default)]
    items: Vec<RemoteTodo>,
    settings: Option<RemoteSettings>,
    #[serde(default)]
    tombstones: Vec<RemoteTombstone>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemotePushResult {
    schema_version: String,
    server_version: i64,
    #[serde(default)]
    accepted: RemoteSnapshot,
    settings: Option<RemoteSettings>,
    #[serde(default)]
    tombstones: Vec<RemoteTombstone>,
    #[serde(default)]
    conflicts: Vec<RemoteConflict>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteChecklist {
    local_id: String,
    id: Option<String>,
    owner_user_id: String,
    sort_index: i32,
    name: String,
    created_at_millis: i64,
    updated_at_millis: i64,
    remote_version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteTodo {
    local_id: String,
    id: Option<String>,
    owner_user_id: String,
    checklist_local_id: String,
    sort_index: i32,
    title: String,
    priority: String,
    due_at_millis: i64,
    completed: bool,
    created_at_millis: i64,
    updated_at_millis: i64,
    reminder_repeat: String,
    image_local_name: Option<String>,
    image_remote_path: Option<String>,
    image_sync_state: String,
    trashed_from_checklist_id: Option<String>,
    trashed_from_checklist_name: Option<String>,
    trashed_at_millis: Option<i64>,
    remote_version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteSettings {
    owner_user_id: Option<String>,
    language_mode: String,
    updated_at_millis: i64,
    remote_version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteTombstone {
    owner_user_id: Option<String>,
    record_type: String,
    local_id: String,
    deleted_at_millis: i64,
    remote_version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteConflict {
    record_type: String,
    local_id: String,
    message: String,
}

pub async fn run_sync(
    cloud: &SupabaseClient,
    repository: &SqliteRepository,
    session: &AuthSession,
    snapshot: &mut AppSnapshot,
) -> Result<SyncRunView, AppError> {
    let cursor = repository.sync_cursor(&session.user_id).await?;
    let dirty = repository.dirty_records().await?;
    let dirty_keys = dirty
        .iter()
        .map(|record| format!("{}:{}", record.record_type, record.local_id))
        .collect::<HashSet<_>>();
    let pulled: RemoteChangeBatch = cloud
        .rpc(
            session,
            "pixeldone_pull_changes",
            json!({ "p_since_version": cursor }),
        )
        .await?;
    require_schema(&pulled.schema_version)?;
    apply_tombstones(repository, snapshot, &pulled.tombstones).await?;
    apply_remote_changes(repository, snapshot, &pulled, &dirty_keys).await?;
    repository.save_snapshot(snapshot).await?;
    repository
        .save_sync_cursor(&session.user_id, pulled.server_version)
        .await?;

    let dirty = repository.dirty_records().await?;
    let tombstones = repository.tombstones().await?;
    let checklists = dirty
        .iter()
        .filter(|record| record.record_type == "checklist")
        .filter_map(|record| {
            snapshot
                .checklists
                .iter()
                .find(|list| list.id == record.local_id)
        })
        .filter(|list| list.kind == ChecklistKind::Normal)
        .map(|list| remote_checklist(snapshot, list, &session.user_id))
        .collect::<Vec<_>>();
    let items = dirty
        .iter()
        .filter(|record| record.record_type == "item")
        .filter_map(|record| find_todo(snapshot, &record.local_id))
        .map(|(list_id, index, item)| remote_todo(list_id, index, item, &session.user_id))
        .collect::<Vec<_>>();
    let settings = if dirty.iter().any(|record| record.record_type == "settings") {
        Some(RemoteSettings {
            owner_user_id: Some(session.user_id.clone()),
            language_mode: snapshot.settings.language_mode.sync_value().to_owned(),
            updated_at_millis: unix_now_millis(),
            remote_version: repository.language_remote_version().await?,
        })
    } else {
        None
    };

    if checklists.is_empty() && items.is_empty() && settings.is_none() && tombstones.is_empty() {
        return sync_view(repository, pulled.server_version, None).await;
    }

    let payload_tombstones = tombstones
        .iter()
        .map(|item| remote_tombstone(item, &session.user_id))
        .collect::<Vec<_>>();
    let pushed: RemotePushResult = cloud
        .rpc(
            session,
            "pixeldone_apply_mutation",
            json!({
                "p_mutation_uuid": Uuid::new_v4().to_string(),
                "p_checklists": checklists,
                "p_items": items,
                "p_settings": settings,
                "p_tombstones": payload_tombstones,
            }),
        )
        .await?;
    require_schema(&pushed.schema_version)?;

    let conflict_keys = pushed
        .conflicts
        .iter()
        .map(|value| format!("{}:{}", value.record_type, value.local_id))
        .collect::<HashSet<_>>();
    for accepted in &pushed.accepted.checklists {
        apply_remote_checklist(snapshot, accepted);
        repository
            .clear_dirty("checklist", &accepted.local_id)
            .await?;
    }
    for accepted in &pushed.accepted.items {
        apply_remote_todo(snapshot, accepted)?;
        repository.clear_dirty("item", &accepted.local_id).await?;
    }
    if let Some(settings) = &pushed.settings {
        snapshot.settings.language_mode = AppLanguage::from_sync_value(&settings.language_mode);
        repository
            .set_language_remote_version(settings.remote_version)
            .await?;
        repository.clear_dirty("settings", "language").await?;
    }
    for accepted in &pushed.tombstones {
        repository
            .delete_tombstone(&accepted.record_type, &accepted.local_id)
            .await?;
    }
    if !conflict_keys.is_empty() {
        let latest: RemoteChangeBatch = cloud
            .rpc(
                session,
                "pixeldone_pull_changes",
                json!({ "p_since_version": 0 }),
            )
            .await?;
        for conflict in &pushed.conflicts {
            save_push_conflict(repository, snapshot, conflict, &latest).await?;
        }
    }
    repository.save_snapshot(snapshot).await?;
    repository
        .save_sync_cursor(&session.user_id, pushed.server_version)
        .await?;
    sync_view(repository, pushed.server_version, None).await
}

pub async fn conflicts(repository: &SqliteRepository) -> Result<Vec<SyncConflictView>, AppError> {
    repository
        .conflicts()
        .await?
        .into_iter()
        .map(|value| {
            let local_payload =
                serde_json::from_str(&value.local_payload_json).unwrap_or(Value::Null);
            let cloud_payload =
                serde_json::from_str(&value.remote_payload_json).unwrap_or(Value::Null);
            let fields = serde_json::from_str(&value.fields_json).unwrap_or_default();
            Ok(SyncConflictView {
                record_type: value.record_type,
                local_id: value.local_id,
                title: payload_title(&local_payload),
                fields,
                local_payload,
                cloud_payload,
                message: value.message,
            })
        })
        .collect()
}

pub async fn resolve_conflict(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    record_type: &str,
    local_id: &str,
    keep_cloud: bool,
) -> Result<(), AppError> {
    let conflict = repository
        .conflicts()
        .await?
        .into_iter()
        .find(|value| value.record_type == record_type && value.local_id == local_id)
        .ok_or_else(|| AppError::NotFound(format!("conflict {record_type}:{local_id}")))?;
    if keep_cloud {
        match record_type {
            "checklist" => {
                let remote: RemoteChecklist =
                    serde_json::from_str(&conflict.remote_payload_json)
                        .map_err(|error| AppError::Database(error.to_string()))?;
                apply_remote_checklist(snapshot, &remote);
            }
            "item" => {
                let remote: RemoteTodo = serde_json::from_str(&conflict.remote_payload_json)
                    .map_err(|error| AppError::Database(error.to_string()))?;
                apply_remote_todo(snapshot, &remote)?;
            }
            "settings" => {
                let remote: RemoteSettings = serde_json::from_str(&conflict.remote_payload_json)
                    .map_err(|error| AppError::Database(error.to_string()))?;
                snapshot.settings.language_mode =
                    AppLanguage::from_sync_value(&remote.language_mode);
                repository
                    .set_language_remote_version(remote.remote_version)
                    .await?;
            }
            _ => return Err(AppError::Validation("未知冲突类型".to_owned())),
        }
        repository.clear_dirty(record_type, local_id).await?;
    } else {
        match record_type {
            "checklist" => {
                let remote: RemoteChecklist =
                    serde_json::from_str(&conflict.remote_payload_json)
                        .map_err(|error| AppError::Database(error.to_string()))?;
                if let Some(local) = snapshot
                    .checklists
                    .iter_mut()
                    .find(|list| list.id == local_id)
                {
                    local.remote_version = remote.remote_version;
                }
            }
            "item" => {
                let remote: RemoteTodo = serde_json::from_str(&conflict.remote_payload_json)
                    .map_err(|error| AppError::Database(error.to_string()))?;
                if let Some((_, _, local)) = find_todo_mut(snapshot, local_id) {
                    local.remote_version = remote.remote_version;
                }
            }
            "settings" => {
                let remote: RemoteSettings = serde_json::from_str(&conflict.remote_payload_json)
                    .map_err(|error| AppError::Database(error.to_string()))?;
                repository
                    .set_language_remote_version(remote.remote_version)
                    .await?;
            }
            _ => return Err(AppError::Validation("未知冲突类型".to_owned())),
        }
    }
    repository.delete_conflict(record_type, local_id).await?;
    repository.save_snapshot(snapshot).await
}

async fn apply_remote_changes(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    batch: &RemoteChangeBatch,
    dirty: &HashSet<String>,
) -> Result<(), AppError> {
    for remote in &batch.checklists {
        let key = format!("checklist:{}", remote.local_id);
        let local = snapshot
            .checklists
            .iter()
            .find(|list| list.id == remote.local_id);
        if dirty.contains(&key)
            && local.and_then(|value| value.remote_version) != remote.remote_version
        {
            save_conflict(
                repository,
                "checklist",
                &remote.local_id,
                local
                    .map(serde_json::to_value)
                    .transpose()?
                    .unwrap_or(Value::Null),
                serde_json::to_value(remote)?,
                "remote_version_changed",
                remote.remote_version,
            )
            .await?;
        } else {
            apply_remote_checklist(snapshot, remote);
            repository
                .clear_dirty("checklist", &remote.local_id)
                .await?;
        }
    }
    for remote in &batch.items {
        let key = format!("item:{}", remote.local_id);
        let local = find_todo(snapshot, &remote.local_id).map(|(_, _, item)| item);
        if dirty.contains(&key)
            && local.and_then(|value| value.remote_version) != remote.remote_version
        {
            save_conflict(
                repository,
                "item",
                &remote.local_id,
                local
                    .map(serde_json::to_value)
                    .transpose()?
                    .unwrap_or(Value::Null),
                serde_json::to_value(remote)?,
                "remote_version_changed",
                remote.remote_version,
            )
            .await?;
        } else {
            apply_remote_todo(snapshot, remote)?;
            repository.clear_dirty("item", &remote.local_id).await?;
        }
    }
    if let Some(remote) = &batch.settings {
        let key = "settings:language";
        let local_version = repository.language_remote_version().await?;
        if dirty.contains(key) && local_version != remote.remote_version {
            save_conflict(
                repository,
                "settings",
                "language",
                json!({
                    "language_mode": snapshot.settings.language_mode.sync_value(),
                    "remote_version": local_version,
                }),
                serde_json::to_value(remote)?,
                "remote_version_changed",
                remote.remote_version,
            )
            .await?;
        } else {
            snapshot.settings.language_mode = AppLanguage::from_sync_value(&remote.language_mode);
            repository
                .set_language_remote_version(remote.remote_version)
                .await?;
            repository.clear_dirty("settings", "language").await?;
        }
    }
    Ok(())
}

async fn apply_tombstones(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    tombstones: &[RemoteTombstone],
) -> Result<(), AppError> {
    for tombstone in tombstones {
        match tombstone.record_type.as_str() {
            "checklist" => snapshot
                .checklists
                .retain(|list| list.kind != ChecklistKind::Normal || list.id != tombstone.local_id),
            "item" => {
                for list in &mut snapshot.checklists {
                    list.items.retain(|item| item.id != tombstone.local_id);
                }
            }
            _ => {}
        }
        repository
            .clear_dirty(&tombstone.record_type, &tombstone.local_id)
            .await?;
    }
    *snapshot = snapshot.clone().normalized();
    Ok(())
}

fn apply_remote_checklist(snapshot: &mut AppSnapshot, remote: &RemoteChecklist) {
    if let Some(local) = snapshot
        .checklists
        .iter_mut()
        .find(|list| list.id == remote.local_id)
    {
        local.name = remote.name.clone();
        local.created_at_millis = remote.created_at_millis;
        local.updated_at_millis = remote.updated_at_millis;
        local.remote_version = remote.remote_version;
        return;
    }
    let mut value = Checklist::new_normal(
        remote.local_id.clone(),
        remote.name.clone(),
        remote.created_at_millis,
    );
    value.updated_at_millis = remote.updated_at_millis;
    value.remote_version = remote.remote_version;
    let special = snapshot
        .checklists
        .iter()
        .position(|list| list.kind != ChecklistKind::Normal)
        .unwrap_or(snapshot.checklists.len());
    let index = usize::try_from(remote.sort_index.max(0))
        .unwrap_or(0)
        .min(special);
    snapshot.checklists.insert(index, value);
}

fn apply_remote_todo(snapshot: &mut AppSnapshot, remote: &RemoteTodo) -> Result<(), AppError> {
    let local_image = find_todo(snapshot, &remote.local_id)
        .and_then(|(_, _, value)| value.image_file_name.clone());
    for list in &mut snapshot.checklists {
        list.items.retain(|item| item.id != remote.local_id);
    }
    if !snapshot
        .checklists
        .iter()
        .any(|list| list.id == remote.checklist_local_id)
    {
        let special = snapshot
            .checklists
            .iter()
            .position(|list| list.kind != ChecklistKind::Normal)
            .unwrap_or(snapshot.checklists.len());
        snapshot.checklists.insert(
            special,
            Checklist::new_normal(
                remote.checklist_local_id.clone(),
                remote
                    .trashed_from_checklist_name
                    .clone()
                    .unwrap_or_else(|| "MAIN".to_owned()),
                remote.created_at_millis,
            ),
        );
    }
    let item = TodoItem {
        id: remote.local_id.clone(),
        title: remote.title.clone(),
        priority: parse_priority(&remote.priority)?,
        due_at_millis: remote.due_at_millis,
        completed: remote.completed,
        created_at_millis: remote.created_at_millis,
        updated_at_millis: remote.updated_at_millis,
        reminder_repeat: parse_repeat(&remote.reminder_repeat)?,
        image_file_name: local_image,
        trashed_from_checklist_id: remote.trashed_from_checklist_id.clone(),
        trashed_from_checklist_name: remote.trashed_from_checklist_name.clone(),
        trashed_at_millis: remote.trashed_at_millis,
        remote_version: remote.remote_version,
    };
    let target = snapshot.checklist_mut(&remote.checklist_local_id)?;
    let index = usize::try_from(remote.sort_index.max(0))
        .unwrap_or(0)
        .min(target.items.len());
    target.items.insert(index, item);
    Ok(())
}

async fn save_push_conflict(
    repository: &SqliteRepository,
    snapshot: &AppSnapshot,
    conflict: &RemoteConflict,
    latest: &RemoteChangeBatch,
) -> Result<(), AppError> {
    let (local, remote, version) = match conflict.record_type.as_str() {
        "checklist" => (
            snapshot
                .checklists
                .iter()
                .find(|value| value.id == conflict.local_id)
                .map(serde_json::to_value)
                .transpose()?
                .unwrap_or(Value::Null),
            latest
                .checklists
                .iter()
                .find(|value| value.local_id == conflict.local_id)
                .map(serde_json::to_value)
                .transpose()?
                .unwrap_or(Value::Null),
            latest
                .checklists
                .iter()
                .find(|value| value.local_id == conflict.local_id)
                .and_then(|value| value.remote_version),
        ),
        "item" => (
            find_todo(snapshot, &conflict.local_id)
                .map(|(_, _, value)| serde_json::to_value(value))
                .transpose()?
                .unwrap_or(Value::Null),
            latest
                .items
                .iter()
                .find(|value| value.local_id == conflict.local_id)
                .map(serde_json::to_value)
                .transpose()?
                .unwrap_or(Value::Null),
            latest
                .items
                .iter()
                .find(|value| value.local_id == conflict.local_id)
                .and_then(|value| value.remote_version),
        ),
        "settings" => (
            json!({ "language_mode": snapshot.settings.language_mode.sync_value() }),
            latest
                .settings
                .as_ref()
                .map(serde_json::to_value)
                .transpose()?
                .unwrap_or(Value::Null),
            latest
                .settings
                .as_ref()
                .and_then(|value| value.remote_version),
        ),
        _ => (Value::Null, Value::Null, None),
    };
    save_conflict(
        repository,
        &conflict.record_type,
        &conflict.local_id,
        local,
        remote,
        &conflict.message,
        version,
    )
    .await
}

async fn save_conflict(
    repository: &SqliteRepository,
    record_type: &str,
    local_id: &str,
    local: Value,
    remote: Value,
    message: &str,
    remote_version: Option<i64>,
) -> Result<(), AppError> {
    let fields = differing_fields(&local, &remote);
    repository
        .save_conflict(&StoredConflict {
            record_type: record_type.to_owned(),
            local_id: local_id.to_owned(),
            local_payload_json: serde_json::to_string(&local)?,
            remote_payload_json: serde_json::to_string(&remote)?,
            fields_json: serde_json::to_string(&fields)?,
            message: message.to_owned(),
            remote_version,
        })
        .await
}

async fn sync_view(
    repository: &SqliteRepository,
    version: i64,
    message: Option<String>,
) -> Result<SyncRunView, AppError> {
    let pending_count =
        repository.dirty_records().await?.len() + repository.tombstones().await?.len();
    let conflict_count = repository.conflicts().await?.len();
    Ok(SyncRunView {
        state: if conflict_count > 0 {
            SyncState::Conflict
        } else {
            SyncState::Synced
        },
        message,
        remote_version: Some(version),
        pending_count,
        conflict_count,
        insecure_http: true,
    })
}

fn remote_checklist(snapshot: &AppSnapshot, list: &Checklist, owner: &str) -> RemoteChecklist {
    RemoteChecklist {
        local_id: list.id.clone(),
        id: None,
        owner_user_id: owner.to_owned(),
        sort_index: snapshot
            .checklists
            .iter()
            .filter(|value| value.kind == ChecklistKind::Normal)
            .position(|value| value.id == list.id)
            .unwrap_or(0) as i32,
        name: list.name.clone(),
        created_at_millis: list.created_at_millis,
        updated_at_millis: list.updated_at_millis,
        remote_version: list.remote_version,
    }
}

fn remote_todo(list_id: &str, index: usize, item: &TodoItem, owner: &str) -> RemoteTodo {
    RemoteTodo {
        local_id: item.id.clone(),
        id: None,
        owner_user_id: owner.to_owned(),
        checklist_local_id: list_id.to_owned(),
        sort_index: index as i32,
        title: item.title.clone(),
        priority: priority_name(item.priority).to_owned(),
        due_at_millis: item.due_at_millis,
        completed: item.completed,
        created_at_millis: item.created_at_millis,
        updated_at_millis: item.updated_at_millis,
        reminder_repeat: repeat_name(item.reminder_repeat).to_owned(),
        image_local_name: None,
        image_remote_path: None,
        image_sync_state: "LOCAL_ONLY".to_owned(),
        trashed_from_checklist_id: item.trashed_from_checklist_id.clone(),
        trashed_from_checklist_name: item.trashed_from_checklist_name.clone(),
        trashed_at_millis: item.trashed_at_millis,
        remote_version: item.remote_version,
    }
}

fn remote_tombstone(value: &LocalTombstone, owner: &str) -> RemoteTombstone {
    RemoteTombstone {
        owner_user_id: Some(owner.to_owned()),
        record_type: value.record_type.clone(),
        local_id: value.local_id.clone(),
        deleted_at_millis: value.deleted_at_millis,
        remote_version: value.remote_version,
    }
}

fn find_todo<'a>(snapshot: &'a AppSnapshot, id: &str) -> Option<(&'a str, usize, &'a TodoItem)> {
    snapshot.checklists.iter().find_map(|list| {
        list.items
            .iter()
            .enumerate()
            .find(|(_, item)| item.id == id)
            .map(|(index, item)| (list.id.as_str(), index, item))
    })
}

fn find_todo_mut<'a>(
    snapshot: &'a mut AppSnapshot,
    id: &str,
) -> Option<(&'a str, usize, &'a mut TodoItem)> {
    snapshot.checklists.iter_mut().find_map(|list| {
        let list_id = list.id.as_str();
        list.items
            .iter_mut()
            .enumerate()
            .find(|(_, item)| item.id == id)
            .map(|(index, item)| (list_id, index, item))
    })
}

fn differing_fields(local: &Value, remote: &Value) -> Vec<String> {
    let ignored = [
        "id",
        "owner_user_id",
        "remote_version",
        "image_local_name",
        "image_remote_path",
        "image_sync_state",
        "imageFileName",
    ];
    let keys = local
        .as_object()
        .into_iter()
        .flat_map(|value| value.keys())
        .chain(
            remote
                .as_object()
                .into_iter()
                .flat_map(|value| value.keys()),
        )
        .filter(|key| !ignored.contains(&key.as_str()))
        .collect::<HashSet<_>>();
    let mut result = keys
        .into_iter()
        .filter(|key| local.get(*key) != remote.get(*key))
        .cloned()
        .collect::<Vec<_>>();
    result.sort();
    result
}

fn payload_title(value: &Value) -> String {
    value
        .get("title")
        .or_else(|| value.get("name"))
        .or_else(|| value.get("language_mode"))
        .and_then(Value::as_str)
        .unwrap_or("SYNC CONFLICT")
        .to_owned()
}

fn require_schema(value: &str) -> Result<(), AppError> {
    if value == EXPECTED_SCHEMA {
        Ok(())
    } else {
        Err(AppError::Network(format!(
            "SERVER UPDATE REQUIRED: expected {EXPECTED_SCHEMA}, received {value}"
        )))
    }
}

fn parse_priority(value: &str) -> Result<TodoPriority, AppError> {
    match value {
        "XHIGH" => Ok(TodoPriority::Xhigh),
        "HIGH" => Ok(TodoPriority::High),
        "MEDIUM" => Ok(TodoPriority::Medium),
        "LOW" => Ok(TodoPriority::Low),
        _ => Err(AppError::Network(format!("未知优先级 {value}"))),
    }
}

fn priority_name(value: TodoPriority) -> &'static str {
    match value {
        TodoPriority::Xhigh => "XHIGH",
        TodoPriority::High => "HIGH",
        TodoPriority::Medium => "MEDIUM",
        TodoPriority::Low => "LOW",
    }
}

fn parse_repeat(value: &str) -> Result<ReminderRepeat, AppError> {
    match value {
        "NONE" => Ok(ReminderRepeat::None),
        "DAILY" => Ok(ReminderRepeat::Daily),
        "WEEKLY" => Ok(ReminderRepeat::Weekly),
        _ => Err(AppError::Network(format!("未知重复提醒 {value}"))),
    }
}

fn repeat_name(value: ReminderRepeat) -> &'static str {
    match value {
        ReminderRepeat::None => "NONE",
        ReminderRepeat::Daily => "DAILY",
        ReminderRepeat::Weekly => "WEEKLY",
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Network(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloud_payload_never_contains_local_image_metadata() {
        let item = TodoItem {
            id: "todo".to_owned(),
            title: "One".to_owned(),
            priority: TodoPriority::High,
            due_at_millis: 10,
            completed: false,
            created_at_millis: 1,
            updated_at_millis: 2,
            reminder_repeat: ReminderRepeat::None,
            image_file_name: Some("private.jpg".to_owned()),
            trashed_from_checklist_id: None,
            trashed_from_checklist_name: None,
            trashed_at_millis: None,
            remote_version: None,
        };
        let remote = remote_todo("main", 0, &item, "owner");
        assert_eq!(remote.image_local_name, None);
        assert_eq!(remote.image_remote_path, None);
        assert_eq!(remote.image_sync_state, "LOCAL_ONLY");
    }
}
