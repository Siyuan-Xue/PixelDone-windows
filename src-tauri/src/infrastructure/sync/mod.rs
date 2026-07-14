use std::{collections::HashSet, fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    domain::{
        AppError, AppLanguage, AppSnapshot, Checklist, ChecklistKind, ReminderRepeat,
        SETTINGS_CHECKLIST_ID, SyncConflictFieldView, SyncConflictValueView, SyncConflictView,
        SyncRunView, SyncState, TRASH_CHECKLIST_ID, TodoItem, TodoPriority, unix_now_millis,
    },
    infrastructure::{
        auth::{AuthSession, SupabaseClient},
        db::{LocalTodoAttachment, LocalTombstone, SqliteRepository, StoredConflict},
        image::{
            cache_file_name, extension_for_content_type, inspect_image_file, is_safe_object_path,
            safe_file_name,
        },
    },
};

const EXPECTED_SCHEMA: &str = "3.2";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct RemoteSnapshot {
    #[serde(default)]
    checklists: Vec<RemoteChecklist>,
    #[serde(default)]
    items: Vec<RemoteTodo>,
    #[serde(default)]
    attachments: Vec<RemoteAttachment>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteChangeBatch {
    schema_version: String,
    server_version: i64,
    #[serde(default)]
    checklists: Vec<RemoteChecklist>,
    #[serde(default)]
    items: Vec<RemoteTodo>,
    #[serde(default)]
    attachments: Vec<RemoteAttachment>,
    settings: Option<RemoteSettings>,
    #[serde(default)]
    tombstones: Vec<RemoteTombstone>,
    #[serde(default)]
    image_cleanup_paths: Vec<String>,
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
    #[serde(default)]
    image_cleanup_paths: Vec<String>,
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
    trashed_from_checklist_id: Option<String>,
    trashed_from_checklist_name: Option<String>,
    trashed_at_millis: Option<i64>,
    remote_version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RemoteAttachment {
    owner_user_id: String,
    todo_local_id: String,
    attachment_id: Option<String>,
    object_path: Option<String>,
    content_sha256: Option<String>,
    content_type: Option<String>,
    byte_size: Option<i64>,
    updated_at_millis: i64,
    deleted_at_millis: Option<i64>,
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

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct MutationPayload {
    #[serde(default)]
    checklists: Vec<RemoteChecklist>,
    #[serde(default)]
    items: Vec<RemoteTodo>,
    #[serde(default)]
    attachments: Vec<RemoteAttachment>,
    settings: Option<RemoteSettings>,
    #[serde(default)]
    tombstones: Vec<RemoteTombstone>,
    #[serde(default)]
    cleaned_image_paths: Vec<String>,
}

pub async fn run_sync(
    cloud: &SupabaseClient,
    repository: &SqliteRepository,
    session: &AuthSession,
    snapshot: &mut AppSnapshot,
    attachments_dir: &Path,
) -> Result<SyncRunView, AppError> {
    repository.delete_synthetic_checklist_conflicts().await?;
    let cursor = repository.sync_cursor(&session.user_id).await?;
    let pull_from = if repository.pristine_initialized(&session.user_id).await? {
        cursor
    } else {
        0
    };
    let dirty = repository.dirty_records().await?;
    let dirty_keys = dirty
        .iter()
        .map(|record| format!("{}:{}", record.record_type, record.local_id))
        .collect::<HashSet<_>>();
    let pulled: RemoteChangeBatch = cloud
        .rpc(
            session,
            "pixeldone_pull_changes",
            json!({
                "p_since_version": pull_from,
                "p_client_schema_version": EXPECTED_SCHEMA,
            }),
        )
        .await?;
    require_schema(&pulled.schema_version)?;
    let blocked_keys = repository
        .conflicts()
        .await?
        .into_iter()
        .map(|conflict| format!("{}:{}", conflict.record_type, conflict.local_id))
        .collect::<HashSet<_>>();
    apply_tombstones(
        repository,
        snapshot,
        &session.user_id,
        &pulled.tombstones,
        attachments_dir,
    )
    .await?;
    apply_remote_changes(
        repository,
        snapshot,
        &session.user_id,
        &pulled,
        &dirty_keys,
        &blocked_keys,
    )
    .await?;
    apply_remote_attachments(repository, snapshot, &pulled.attachments, attachments_dir).await?;
    drain_local_cleanup_queue(cloud, repository, session).await?;
    repository.save_snapshot(snapshot).await?;
    repository
        .save_sync_cursor(&session.user_id, pulled.server_version)
        .await?;
    repository
        .mark_pristine_initialized(&session.user_id)
        .await?;

    let cleaned_image_paths =
        clean_remote_objects(cloud, session, &pulled.image_cleanup_paths).await;
    prepare_pending_attachments(cloud, repository, session, snapshot, attachments_dir).await?;

    let dirty = repository.dirty_records().await?;
    let blocked_keys = repository
        .conflicts()
        .await?
        .into_iter()
        .map(|conflict| format!("{}:{}", conflict.record_type, conflict.local_id))
        .collect::<HashSet<_>>();
    let tombstones = repository.tombstones().await?;
    let checklists = dirty
        .iter()
        .filter(|record| record.record_type == "checklist")
        .filter(|record| !blocked_keys.contains(&format!("checklist:{}", record.local_id)))
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
        .filter(|record| !blocked_keys.contains(&format!("item:{}", record.local_id)))
        .filter_map(|record| find_todo(snapshot, &record.local_id))
        .map(|(list_id, index, item)| remote_todo(list_id, index, item, &session.user_id))
        .collect::<Vec<_>>();
    let settings = if dirty.iter().any(|record| record.record_type == "settings")
        && !blocked_keys.contains("settings:language")
    {
        Some(RemoteSettings {
            owner_user_id: Some(session.user_id.clone()),
            language_mode: snapshot.settings.language_mode.sync_value().to_owned(),
            updated_at_millis: unix_now_millis(),
            remote_version: repository.language_remote_version().await?,
        })
    } else {
        None
    };
    let attachments = repository
        .attachments()
        .await?
        .into_iter()
        .filter(|value| value.sync_state == "METADATA_PENDING")
        .map(|value| remote_attachment(value, &session.user_id))
        .collect::<Vec<_>>();

    let payload_tombstones = tombstones
        .iter()
        .filter(|item| !blocked_keys.contains(&format!("{}:{}", item.record_type, item.local_id)))
        .map(|item| remote_tombstone(item, &session.user_id))
        .collect::<Vec<_>>();
    let new_payload = MutationPayload {
        checklists,
        items,
        attachments,
        settings,
        tombstones: payload_tombstones,
        cleaned_image_paths,
    };
    let pending = repository.pending_mutation(&session.user_id).await?;
    let (mutation_uuid, payload) = if let Some(stored) = pending {
        (
            stored.mutation_uuid,
            serde_json::from_str::<MutationPayload>(&stored.payload_json)
                .map_err(|error| AppError::Database(error.to_string()))?,
        )
    } else if mutation_payload_is_empty(&new_payload) {
        return sync_view(repository, pulled.server_version, None).await;
    } else {
        let mutation_uuid = Uuid::new_v4().to_string();
        repository
            .save_pending_mutation(
                &session.user_id,
                &mutation_uuid,
                pulled.server_version,
                &serde_json::to_string(&new_payload)?,
            )
            .await?;
        (mutation_uuid, new_payload)
    };
    let pushed: RemotePushResult = match cloud
        .rpc(
            session,
            "pixeldone_apply_mutation",
            mutation_request(&mutation_uuid, &payload),
        )
        .await
    {
        Ok(value) => value,
        Err(error) => {
            repository
                .mark_mutation_attempt(&session.user_id, &mutation_uuid, Some(&error.to_string()))
                .await?;
            return Err(error);
        }
    };
    repository
        .mark_mutation_attempt(&session.user_id, &mutation_uuid, None)
        .await?;
    require_schema(&pushed.schema_version)?;

    let conflict_keys = pushed
        .conflicts
        .iter()
        .map(|value| format!("{}:{}", value.record_type, value.local_id))
        .collect::<HashSet<_>>();
    for accepted in &pushed.accepted.checklists {
        let changed_after_send = payload
            .checklists
            .iter()
            .find(|sent| sent.local_id == accepted.local_id)
            .and_then(|sent| {
                snapshot
                    .checklists
                    .iter()
                    .find(|list| list.id == accepted.local_id)
                    .map(|current| {
                        let current = serde_json::to_value(remote_checklist(
                            snapshot,
                            current,
                            &session.user_id,
                        ))
                        .expect("checklist sync payload must serialize");
                        let sent = serde_json::to_value(sent)
                            .expect("stored checklist payload must serialize");
                        !semantic_differences("checklist", &current, &sent).is_empty()
                    })
            })
            .unwrap_or(false);
        if changed_after_send {
            if let Some(local) = snapshot
                .checklists
                .iter_mut()
                .find(|list| list.id == accepted.local_id)
            {
                local.remote_version = accepted.remote_version;
            }
            repository
                .mark_dirty("checklist", &accepted.local_id)
                .await?;
        } else {
            apply_remote_checklist(snapshot, accepted);
            repository
                .clear_dirty("checklist", &accepted.local_id)
                .await?;
        }
        save_pristine(
            repository,
            &session.user_id,
            "checklist",
            &accepted.local_id,
            accepted,
        )
        .await?;
    }
    for accepted in &pushed.accepted.items {
        let changed_after_send = payload
            .items
            .iter()
            .find(|sent| sent.local_id == accepted.local_id)
            .and_then(|sent| {
                find_todo(snapshot, &accepted.local_id).map(|(list_id, index, current)| {
                    let current = serde_json::to_value(remote_todo(
                        list_id,
                        index,
                        current,
                        &session.user_id,
                    ))
                    .expect("todo sync payload must serialize");
                    let sent =
                        serde_json::to_value(sent).expect("stored todo payload must serialize");
                    !semantic_differences("item", &current, &sent).is_empty()
                })
            })
            .unwrap_or(false);
        if changed_after_send {
            if let Some((_, _, local)) = find_todo_mut(snapshot, &accepted.local_id) {
                local.remote_version = accepted.remote_version;
            }
            repository.mark_dirty("item", &accepted.local_id).await?;
        } else {
            apply_remote_todo(snapshot, accepted)?;
            repository.clear_dirty("item", &accepted.local_id).await?;
        }
        save_pristine(
            repository,
            &session.user_id,
            "item",
            &accepted.local_id,
            accepted,
        )
        .await?;
    }
    apply_accepted_attachments(
        repository,
        snapshot,
        &pushed.accepted.attachments,
        attachments_dir,
    )
    .await?;
    if let Some(settings) = &pushed.settings {
        let changed_after_send = payload
            .settings
            .as_ref()
            .is_some_and(|sent| snapshot.settings.language_mode.sync_value() != sent.language_mode);
        repository
            .set_language_remote_version(settings.remote_version)
            .await?;
        if changed_after_send {
            repository.mark_dirty("settings", "language").await?;
        } else {
            snapshot.settings.language_mode = AppLanguage::from_sync_value(&settings.language_mode);
            repository.clear_dirty("settings", "language").await?;
        }
        save_pristine(
            repository,
            &session.user_id,
            "settings",
            "language",
            settings,
        )
        .await?;
    }
    for accepted in &pushed.tombstones {
        repository
            .delete_tombstone(&accepted.record_type, &accepted.local_id)
            .await?;
        repository
            .delete_pristine_record(&session.user_id, &accepted.record_type, &accepted.local_id)
            .await?;
        if accepted.record_type == "item" {
            delete_local_attachment(repository, attachments_dir, &accepted.local_id).await?;
        }
    }
    drain_local_cleanup_queue(cloud, repository, session).await?;
    if !conflict_keys.is_empty() {
        let latest: RemoteChangeBatch = cloud
            .rpc(
                session,
                "pixeldone_pull_changes",
                json!({
                    "p_since_version": 0,
                    "p_client_schema_version": EXPECTED_SCHEMA,
                }),
            )
            .await?;
        apply_remote_attachments(repository, snapshot, &latest.attachments, attachments_dir)
            .await?;
        drain_local_cleanup_queue(cloud, repository, session).await?;
        for conflict in &pushed.conflicts {
            if conflict.record_type != "attachment" {
                save_push_conflict(repository, snapshot, &session.user_id, conflict, &latest)
                    .await?;
            }
        }
    }

    let cleaned_after_push =
        clean_remote_objects(cloud, session, &pushed.image_cleanup_paths).await;
    if !cleaned_after_push.is_empty() {
        let acknowledged: RemotePushResult = cloud
            .rpc(
                session,
                "pixeldone_apply_mutation",
                json!({
                    "p_mutation_uuid": Uuid::new_v4().to_string(),
                    "p_client_schema_version": EXPECTED_SCHEMA,
                    "p_checklists": [],
                    "p_items": [],
                    "p_attachments": [],
                    "p_settings": null,
                    "p_tombstones": [],
                    "p_cleaned_image_paths": cleaned_after_push,
                }),
            )
            .await?;
        require_schema(&acknowledged.schema_version)?;
    }
    repository.save_snapshot(snapshot).await?;
    repository
        .delete_pending_mutation(&session.user_id, &mutation_uuid)
        .await?;
    repository
        .save_sync_cursor(&session.user_id, pushed.server_version)
        .await?;
    sync_view(repository, pushed.server_version, None).await
}

pub async fn conflicts(repository: &SqliteRepository) -> Result<Vec<SyncConflictView>, AppError> {
    repository.delete_synthetic_checklist_conflicts().await?;
    let snapshot = repository
        .load_snapshot()
        .await?
        .unwrap_or_else(|| AppSnapshot::initial(unix_now_millis()));
    let mut result = Vec::new();
    for conflict in repository.conflicts().await? {
        let local = serde_json::from_str(&conflict.local_payload_json).unwrap_or(Value::Null);
        let cloud = serde_json::from_str(&conflict.remote_payload_json).unwrap_or(Value::Null);
        let field_keys =
            serde_json::from_str::<Vec<String>>(&conflict.fields_json).unwrap_or_default();
        let fields = field_keys
            .into_iter()
            .map(|key| SyncConflictFieldView {
                local_value: conflict_value(&key, &local, &snapshot),
                cloud_value: conflict_value(&key, &cloud, &snapshot),
                key,
            })
            .collect();
        result.push(SyncConflictView {
            record_type: conflict.record_type,
            local_id: conflict.local_id,
            title: conflict_title(&local, &cloud),
            fields,
            message_code: conflict.message,
        });
    }
    Ok(result)
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
    let local = serde_json::from_str::<Value>(&conflict.local_payload_json)
        .map_err(|error| AppError::Database(error.to_string()))?;
    let cloud = serde_json::from_str::<Value>(&conflict.remote_payload_json)
        .map_err(|error| AppError::Database(error.to_string()))?;
    let fields = serde_json::from_str::<Vec<String>>(&conflict.fields_json)
        .map_err(|error| AppError::Database(error.to_string()))?;
    let mut selected = local;
    if keep_cloud {
        for field in &fields {
            if let Some(object) = selected.as_object_mut() {
                object.insert(
                    field.clone(),
                    cloud.get(field).cloned().unwrap_or(Value::Null),
                );
            }
        }
    }
    if let Some(object) = selected.as_object_mut() {
        object.insert(
            "remote_version".to_owned(),
            cloud.get("remote_version").cloned().unwrap_or(Value::Null),
        );
    }
    let cloud_version = cloud.get("remote_version").and_then(Value::as_i64);
    apply_merged_record(repository, snapshot, record_type, &selected, cloud_version).await?;
    if semantic_differences(record_type, &selected, &cloud).is_empty() {
        repository.clear_dirty(record_type, local_id).await?;
    } else {
        repository.mark_dirty(record_type, local_id).await?;
    }
    let owner = cloud
        .get("owner_user_id")
        .and_then(Value::as_str)
        .unwrap_or("current");
    save_pristine_value(
        repository,
        owner,
        record_type,
        local_id,
        &cloud,
        cloud_version,
    )
    .await?;
    repository.delete_conflict(record_type, local_id).await?;
    repository.save_snapshot(snapshot).await
}

async fn apply_remote_changes(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    owner: &str,
    batch: &RemoteChangeBatch,
    dirty: &HashSet<String>,
    blocked: &HashSet<String>,
) -> Result<(), AppError> {
    for remote in &batch.checklists {
        if is_synthetic_checklist_id(&remote.local_id) {
            continue;
        }
        let key = format!("checklist:{}", remote.local_id);
        if blocked.contains(&key) {
            continue;
        }
        let local_value = snapshot
            .checklists
            .iter()
            .find(|list| list.id == remote.local_id)
            .map(|list| serde_json::to_value(remote_checklist(snapshot, list, owner)))
            .transpose()?
            .unwrap_or(Value::Null);
        let remote_value = serde_json::to_value(remote)?;
        if !dirty.contains(&key) || local_value.is_null() {
            apply_remote_checklist(snapshot, remote);
            repository
                .clear_dirty("checklist", &remote.local_id)
                .await?;
            save_pristine(repository, owner, "checklist", &remote.local_id, remote).await?;
            continue;
        }
        merge_remote_record(
            repository,
            snapshot,
            owner,
            "checklist",
            &remote.local_id,
            local_value,
            remote_value,
        )
        .await?;
    }
    for remote in &batch.items {
        let key = format!("item:{}", remote.local_id);
        if blocked.contains(&key) {
            continue;
        }
        let local_value = find_todo(snapshot, &remote.local_id)
            .map(|(list_id, index, item)| {
                serde_json::to_value(remote_todo(list_id, index, item, owner))
            })
            .transpose()?
            .unwrap_or(Value::Null);
        let remote_value = serde_json::to_value(remote)?;
        if !dirty.contains(&key) || local_value.is_null() {
            apply_remote_todo(snapshot, remote)?;
            repository.clear_dirty("item", &remote.local_id).await?;
            save_pristine(repository, owner, "item", &remote.local_id, remote).await?;
            continue;
        }
        merge_remote_record(
            repository,
            snapshot,
            owner,
            "item",
            &remote.local_id,
            local_value,
            remote_value,
        )
        .await?;
    }
    if let Some(remote) = &batch.settings {
        let key = "settings:language";
        if blocked.contains(key) {
            return Ok(());
        }
        let local_version = repository.language_remote_version().await?;
        let local_value = serde_json::to_value(RemoteSettings {
            owner_user_id: Some(owner.to_owned()),
            language_mode: snapshot.settings.language_mode.sync_value().to_owned(),
            updated_at_millis: remote.updated_at_millis,
            remote_version: local_version,
        })?;
        if !dirty.contains(key) {
            snapshot.settings.language_mode = AppLanguage::from_sync_value(&remote.language_mode);
            repository
                .set_language_remote_version(remote.remote_version)
                .await?;
            repository.clear_dirty("settings", "language").await?;
            save_pristine(repository, owner, "settings", "language", remote).await?;
        } else {
            merge_remote_record(
                repository,
                snapshot,
                owner,
                "settings",
                "language",
                local_value,
                serde_json::to_value(remote)?,
            )
            .await?;
        }
    }
    Ok(())
}

struct MergeDecision {
    merged: Value,
    conflict_fields: Vec<String>,
    needs_push: bool,
}

async fn merge_remote_record(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    owner: &str,
    record_type: &str,
    local_id: &str,
    local: Value,
    cloud: Value,
) -> Result<(), AppError> {
    let cloud_version = cloud.get("remote_version").and_then(Value::as_i64);
    let pristine = repository
        .pristine_record(owner, record_type, local_id)
        .await?;
    let decision = if let Some(pristine) = pristine {
        let base = serde_json::from_str::<Value>(&pristine.payload_json)
            .map_err(|error| AppError::Database(error.to_string()))?;
        three_way_merge(record_type, &base, &local, &cloud)
    } else {
        let local_version = local.get("remote_version").and_then(Value::as_i64);
        let fields = semantic_differences(record_type, &local, &cloud);
        if local_version == cloud_version {
            MergeDecision {
                merged: local.clone(),
                conflict_fields: Vec::new(),
                needs_push: !fields.is_empty(),
            }
        } else if fields.is_empty() {
            MergeDecision {
                merged: cloud.clone(),
                conflict_fields: Vec::new(),
                needs_push: false,
            }
        } else {
            MergeDecision {
                merged: local.clone(),
                conflict_fields: fields,
                needs_push: true,
            }
        }
    };

    apply_merged_record(
        repository,
        snapshot,
        record_type,
        &decision.merged,
        cloud_version,
    )
    .await?;
    save_pristine_value(
        repository,
        owner,
        record_type,
        local_id,
        &cloud,
        cloud_version,
    )
    .await?;

    if decision.conflict_fields.is_empty() {
        repository.delete_conflict(record_type, local_id).await?;
        if decision.needs_push {
            repository.mark_dirty(record_type, local_id).await?;
        } else {
            repository.clear_dirty(record_type, local_id).await?;
        }
    } else {
        repository.mark_dirty(record_type, local_id).await?;
        save_conflict_with_fields(
            repository,
            record_type,
            local_id,
            decision.merged,
            cloud,
            "overlapping_fields_changed",
            cloud_version,
            &decision.conflict_fields,
        )
        .await?;
    }
    Ok(())
}

async fn apply_merged_record(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    record_type: &str,
    value: &Value,
    cloud_version: Option<i64>,
) -> Result<(), AppError> {
    match record_type {
        "checklist" => {
            let remote: RemoteChecklist = serde_json::from_value(value.clone())?;
            apply_remote_checklist(snapshot, &remote);
        }
        "item" => {
            let remote: RemoteTodo = serde_json::from_value(value.clone())?;
            apply_remote_todo(snapshot, &remote)?;
        }
        "settings" => {
            let remote: RemoteSettings = serde_json::from_value(value.clone())?;
            snapshot.settings.language_mode = AppLanguage::from_sync_value(&remote.language_mode);
            repository
                .set_language_remote_version(cloud_version)
                .await?;
        }
        _ => {
            return Err(AppError::Validation(format!(
                "Unknown sync record type {record_type}"
            )));
        }
    }
    Ok(())
}

fn three_way_merge(record_type: &str, base: &Value, local: &Value, cloud: &Value) -> MergeDecision {
    let mut merged = cloud.clone();
    let mut conflict_fields = Vec::new();
    let mut needs_push = false;
    for field in sync_fields(record_type) {
        let local_changed = base.get(*field) != local.get(*field);
        let cloud_changed = base.get(*field) != cloud.get(*field);
        if local_changed && local.get(*field) != cloud.get(*field) {
            needs_push = true;
            if let Some(object) = merged.as_object_mut() {
                object.insert(
                    (*field).to_owned(),
                    local.get(*field).cloned().unwrap_or(Value::Null),
                );
            }
        }
        if local_changed && cloud_changed && local.get(*field) != cloud.get(*field) {
            conflict_fields.push((*field).to_owned());
        }
    }
    MergeDecision {
        merged,
        conflict_fields,
        needs_push,
    }
}

fn semantic_differences(record_type: &str, local: &Value, cloud: &Value) -> Vec<String> {
    sync_fields(record_type)
        .iter()
        .filter(|field| local.get(**field) != cloud.get(**field))
        .map(|field| (*field).to_owned())
        .collect()
}

fn sync_fields(record_type: &str) -> &'static [&'static str] {
    match record_type {
        "checklist" => &["sort_index", "name"],
        "item" => &[
            "checklist_local_id",
            "sort_index",
            "title",
            "priority",
            "due_at_millis",
            "completed",
            "reminder_repeat",
            "trashed_from_checklist_id",
            "trashed_from_checklist_name",
            "trashed_at_millis",
        ],
        "settings" => &["language_mode"],
        _ => &[],
    }
}

fn is_synthetic_checklist_id(local_id: &str) -> bool {
    matches!(local_id, TRASH_CHECKLIST_ID | SETTINGS_CHECKLIST_ID)
}

async fn apply_tombstones(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    owner: &str,
    tombstones: &[RemoteTombstone],
    attachments_dir: &Path,
) -> Result<(), AppError> {
    for tombstone in tombstones {
        if tombstone.record_type == "checklist" && is_synthetic_checklist_id(&tombstone.local_id) {
            repository
                .clear_dirty(&tombstone.record_type, &tombstone.local_id)
                .await?;
            continue;
        }
        match tombstone.record_type.as_str() {
            "checklist" => {
                let todo_ids = snapshot
                    .checklists
                    .iter()
                    .find(|list| list.id == tombstone.local_id)
                    .map(|list| {
                        list.items
                            .iter()
                            .map(|item| item.id.clone())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                for todo_id in todo_ids {
                    delete_local_attachment(repository, attachments_dir, &todo_id).await?;
                }
                snapshot.checklists.retain(|list| {
                    list.kind != ChecklistKind::Normal || list.id != tombstone.local_id
                });
            }
            "item" => {
                delete_local_attachment(repository, attachments_dir, &tombstone.local_id).await?;
                for list in &mut snapshot.checklists {
                    list.items.retain(|item| item.id != tombstone.local_id);
                }
            }
            _ => {}
        }
        repository
            .clear_dirty(&tombstone.record_type, &tombstone.local_id)
            .await?;
        repository
            .delete_pristine_record(owner, &tombstone.record_type, &tombstone.local_id)
            .await?;
    }
    *snapshot = snapshot.clone().normalized();
    Ok(())
}

async fn prepare_pending_attachments(
    cloud: &SupabaseClient,
    repository: &SqliteRepository,
    session: &AuthSession,
    snapshot: &AppSnapshot,
    attachments_dir: &Path,
) -> Result<(), AppError> {
    let active_todo_ids = snapshot
        .checklists
        .iter()
        .flat_map(|list| list.items.iter())
        .map(|item| item.id.as_str())
        .collect::<HashSet<_>>();
    for mut attachment in repository.attachments().await?.into_iter().filter(|value| {
        active_todo_ids.contains(value.todo_id.as_str())
            && value.deleted_at_millis.is_none()
            && matches!(value.sync_state.as_str(), "PENDING_UPLOAD" | "ERROR")
    }) {
        let Some(file_name) = attachment.local_file_name.clone() else {
            attachment.sync_state = "ERROR".to_owned();
            attachment.last_error = Some("Local image file is missing".to_owned());
            repository.save_attachment(&attachment).await?;
            continue;
        };
        if !safe_file_name(&file_name) {
            attachment.sync_state = "ERROR".to_owned();
            attachment.last_error = Some("Invalid local image file name".to_owned());
            repository.save_attachment(&attachment).await?;
            continue;
        }
        let (bytes, metadata) = match inspect_image_file(&attachments_dir.join(&file_name)) {
            Ok(value) => value,
            Err(error) => {
                attachment.sync_state = "ERROR".to_owned();
                attachment.last_error = Some(error.to_string().chars().take(280).collect());
                repository.save_attachment(&attachment).await?;
                continue;
            }
        };
        let attachment_id = attachment
            .attachment_id
            .clone()
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let extension = extension_for_content_type(metadata.content_type)?;
        let object_path = format!(
            "{}/{}/{}-{}.{}",
            session.user_id, attachment.todo_id, attachment_id, metadata.content_sha256, extension
        );
        if !is_safe_object_path(&object_path) {
            attachment.sync_state = "ERROR".to_owned();
            attachment.last_error = Some("Invalid Storage object path".to_owned());
            repository.save_attachment(&attachment).await?;
            continue;
        }
        match cloud
            .upload_todo_image(session, &object_path, metadata.content_type, bytes)
            .await
        {
            Ok(()) => {
                attachment.attachment_id = Some(attachment_id);
                attachment.object_path = Some(object_path);
                attachment.content_sha256 = Some(metadata.content_sha256);
                attachment.content_type = Some(metadata.content_type.to_owned());
                attachment.byte_size = Some(metadata.byte_size);
                attachment.sync_state = "METADATA_PENDING".to_owned();
                attachment.last_error = None;
            }
            Err(error) => {
                attachment.sync_state = "ERROR".to_owned();
                attachment.last_error = Some(error.to_string().chars().take(280).collect());
            }
        }
        repository.save_attachment(&attachment).await?;
    }
    Ok(())
}

async fn clean_remote_objects(
    cloud: &SupabaseClient,
    session: &AuthSession,
    paths: &[String],
) -> Vec<String> {
    let mut cleaned = Vec::new();
    for path in paths.iter().filter(|path| is_safe_object_path(path)) {
        if cloud.delete_todo_image_object(session, path).await.is_ok() {
            cleaned.push(path.clone());
        }
    }
    cleaned
}

async fn drain_local_cleanup_queue(
    cloud: &SupabaseClient,
    repository: &SqliteRepository,
    session: &AuthSession,
) -> Result<(), AppError> {
    for (todo_id, object_path) in repository.local_image_cleanup_paths().await? {
        let current = repository.attachment(&todo_id).await?;
        let still_pending = current.as_ref().is_some_and(|value| {
            matches!(
                value.sync_state.as_str(),
                "PENDING_UPLOAD" | "METADATA_PENDING" | "ERROR"
            )
        });
        let still_current = current
            .as_ref()
            .and_then(|value| value.object_path.as_deref())
            == Some(object_path.as_str());
        if still_pending || still_current || !is_safe_object_path(&object_path) {
            continue;
        }
        if cloud
            .delete_todo_image_object(session, &object_path)
            .await
            .is_ok()
        {
            repository
                .delete_local_image_cleanup_path(&todo_id, &object_path)
                .await?;
        }
    }
    Ok(())
}

async fn apply_accepted_attachments(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    values: &[RemoteAttachment],
    attachments_dir: &Path,
) -> Result<(), AppError> {
    apply_remote_attachments(repository, snapshot, values, attachments_dir).await
}

async fn apply_remote_attachments(
    repository: &SqliteRepository,
    snapshot: &mut AppSnapshot,
    values: &[RemoteAttachment],
    attachments_dir: &Path,
) -> Result<(), AppError> {
    for remote in values {
        let local = repository.attachment(&remote.todo_local_id).await?;
        if let Some(mut local) = local.clone()
            && matches!(
                local.sync_state.as_str(),
                "PENDING_UPLOAD" | "METADATA_PENDING" | "ERROR"
            )
            && local.updated_at_millis > remote.updated_at_millis
        {
            local.remote_version = remote.remote_version;
            repository.save_attachment(&local).await?;
            continue;
        }

        if let Some(path) = local
            .as_ref()
            .and_then(|value| value.object_path.as_ref())
            .filter(|path| remote.object_path.as_ref() != Some(*path))
        {
            repository
                .queue_local_image_cleanup(&remote.todo_local_id, path)
                .await?;
        }

        if remote.deleted_at_millis.is_some() {
            if let Some(file_name) = local.and_then(|value| value.local_file_name)
                && safe_file_name(&file_name)
            {
                let _ = fs::remove_file(attachments_dir.join(file_name));
            }
            if let Some((_, _, item)) = find_todo_mut(snapshot, &remote.todo_local_id) {
                item.image_file_name = None;
            }
            repository
                .save_attachment(&LocalTodoAttachment {
                    todo_id: remote.todo_local_id.clone(),
                    updated_at_millis: remote.updated_at_millis,
                    deleted_at_millis: remote.deleted_at_millis,
                    remote_version: remote.remote_version,
                    sync_state: "SYNCED".to_owned(),
                    ..LocalTodoAttachment::default()
                })
                .await?;
            continue;
        }

        let attachment_id = required_attachment_field(&remote.attachment_id, "attachment_id")?;
        let object_path = required_attachment_field(&remote.object_path, "object_path")?;
        let hash = required_attachment_field(&remote.content_sha256, "content_sha256")?;
        let content_type = required_attachment_field(&remote.content_type, "content_type")?;
        let byte_size = remote
            .byte_size
            .filter(|size| (1..=10 * 1024 * 1024).contains(size))
            .ok_or_else(|| AppError::Network("Invalid attachment byte_size".to_owned()))?;
        if !is_safe_object_path(object_path) {
            return Err(AppError::Network(
                "Invalid attachment object_path".to_owned(),
            ));
        }
        let expected_prefix = format!("{}/{}/", remote.owner_user_id, remote.todo_local_id);
        if !object_path.starts_with(&expected_prefix) {
            return Err(AppError::Network(
                "Attachment object_path owner prefix does not match metadata".to_owned(),
            ));
        }
        let cache_name = cache_file_name(&remote.todo_local_id, attachment_id, hash, content_type)?;
        let local_match = local
            .as_ref()
            .and_then(|value| value.local_file_name.as_ref())
            .filter(|name| safe_file_name(name))
            .and_then(|name| {
                inspect_image_file(&attachments_dir.join(name))
                    .ok()
                    .filter(|(_, metadata)| {
                        metadata.content_sha256 == hash.as_str()
                            && metadata.content_type == content_type.as_str()
                            && metadata.byte_size == byte_size
                    })
                    .map(|_| name.clone())
            });
        if local_match.is_none()
            && let Some(stale) = local
                .as_ref()
                .and_then(|value| value.local_file_name.as_ref())
                .filter(|name| safe_file_name(name))
        {
            let _ = fs::remove_file(attachments_dir.join(stale));
        }
        let local_file_name = local_match.unwrap_or(cache_name);
        let cached = attachments_dir.join(&local_file_name).is_file();
        if let Some((_, _, item)) = find_todo_mut(snapshot, &remote.todo_local_id) {
            item.image_file_name = Some(local_file_name.clone());
        }
        repository
            .save_attachment(&LocalTodoAttachment {
                todo_id: remote.todo_local_id.clone(),
                local_file_name: Some(local_file_name),
                attachment_id: Some(attachment_id.clone()),
                object_path: Some(object_path.clone()),
                content_sha256: Some(hash.clone()),
                content_type: Some(content_type.clone()),
                byte_size: Some(byte_size),
                updated_at_millis: remote.updated_at_millis,
                deleted_at_millis: None,
                remote_version: remote.remote_version,
                sync_state: if cached { "SYNCED" } else { "REMOTE_ONLY" }.to_owned(),
                last_error: None,
            })
            .await?;
    }
    Ok(())
}

fn required_attachment_field<'a>(
    value: &'a Option<String>,
    name: &str,
) -> Result<&'a String, AppError> {
    value
        .as_ref()
        .ok_or_else(|| AppError::Network(format!("Attachment {name} is missing")))
}

async fn delete_local_attachment(
    repository: &SqliteRepository,
    attachments_dir: &Path,
    todo_id: &str,
) -> Result<(), AppError> {
    if let Some(value) = repository.attachment(todo_id).await? {
        if let Some(file_name) = value.local_file_name
            && safe_file_name(&file_name)
        {
            let _ = fs::remove_file(attachments_dir.join(file_name));
        }
        repository.delete_attachment(todo_id).await?;
    }
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
    owner: &str,
    conflict: &RemoteConflict,
    latest: &RemoteChangeBatch,
) -> Result<(), AppError> {
    let (local, remote, version) = match conflict.record_type.as_str() {
        "checklist" => (
            snapshot
                .checklists
                .iter()
                .find(|value| value.id == conflict.local_id)
                .map(|value| serde_json::to_value(remote_checklist(snapshot, value, owner)))
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
                .map(|(list_id, index, value)| {
                    serde_json::to_value(remote_todo(list_id, index, value, owner))
                })
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
    if !remote.is_null() {
        save_pristine_value(
            repository,
            owner,
            &conflict.record_type,
            &conflict.local_id,
            &remote,
            version,
        )
        .await?;
    }
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
    let fields = semantic_differences(record_type, &local, &remote);
    save_conflict_with_fields(
        repository,
        record_type,
        local_id,
        local,
        remote,
        message,
        remote_version,
        &fields,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn save_conflict_with_fields(
    repository: &SqliteRepository,
    record_type: &str,
    local_id: &str,
    local: Value,
    remote: Value,
    message: &str,
    remote_version: Option<i64>,
    fields: &[String],
) -> Result<(), AppError> {
    repository
        .save_conflict(&StoredConflict {
            record_type: record_type.to_owned(),
            local_id: local_id.to_owned(),
            local_payload_json: serde_json::to_string(&local)?,
            remote_payload_json: serde_json::to_string(&remote)?,
            fields_json: serde_json::to_string(fields)?,
            message: message.to_owned(),
            remote_version,
        })
        .await
}

async fn save_pristine<T: Serialize>(
    repository: &SqliteRepository,
    owner: &str,
    record_type: &str,
    local_id: &str,
    value: &T,
) -> Result<(), AppError> {
    let json = serde_json::to_value(value)?;
    let remote_version = json.get("remote_version").and_then(Value::as_i64);
    save_pristine_value(
        repository,
        owner,
        record_type,
        local_id,
        &json,
        remote_version,
    )
    .await
}

async fn save_pristine_value(
    repository: &SqliteRepository,
    owner: &str,
    record_type: &str,
    local_id: &str,
    value: &Value,
    remote_version: Option<i64>,
) -> Result<(), AppError> {
    repository
        .save_pristine_record(
            owner,
            record_type,
            local_id,
            &serde_json::to_string(value)?,
            remote_version,
        )
        .await
}

fn mutation_payload_is_empty(payload: &MutationPayload) -> bool {
    payload.checklists.is_empty()
        && payload.items.is_empty()
        && payload.attachments.is_empty()
        && payload.settings.is_none()
        && payload.tombstones.is_empty()
        && payload.cleaned_image_paths.is_empty()
}

fn mutation_request(mutation_uuid: &str, payload: &MutationPayload) -> Value {
    json!({
        "p_mutation_uuid": mutation_uuid,
        "p_client_schema_version": EXPECTED_SCHEMA,
        "p_checklists": payload.checklists,
        "p_items": payload.items,
        "p_attachments": payload.attachments,
        "p_settings": payload.settings,
        "p_tombstones": payload.tombstones,
        "p_cleaned_image_paths": payload.cleaned_image_paths,
    })
}

async fn sync_view(
    repository: &SqliteRepository,
    version: i64,
    message: Option<String>,
) -> Result<SyncRunView, AppError> {
    let attachments = repository.attachments().await?;
    let image_error = attachments
        .iter()
        .find(|value| value.sync_state == "ERROR")
        .and_then(|value| value.last_error.clone())
        .map(|error| format!("Todo image sync pending: {error}"));
    let pending_attachments = attachments
        .into_iter()
        .filter(|value| !matches!(value.sync_state.as_str(), "SYNCED" | "REMOTE_ONLY"))
        .count();
    let pending_count = repository.dirty_records().await?.len()
        + repository.tombstones().await?.len()
        + pending_attachments
        + repository.local_image_cleanup_paths().await?.len();
    let conflict_count = repository.conflicts().await?.len();
    Ok(SyncRunView {
        state: if conflict_count > 0 {
            SyncState::Conflict
        } else {
            SyncState::Synced
        },
        message: message.or(image_error),
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
        trashed_from_checklist_id: item.trashed_from_checklist_id.clone(),
        trashed_from_checklist_name: item.trashed_from_checklist_name.clone(),
        trashed_at_millis: item.trashed_at_millis,
        remote_version: item.remote_version,
    }
}

fn remote_attachment(value: LocalTodoAttachment, owner: &str) -> RemoteAttachment {
    RemoteAttachment {
        owner_user_id: owner.to_owned(),
        todo_local_id: value.todo_id,
        attachment_id: value.attachment_id,
        object_path: value.object_path,
        content_sha256: value.content_sha256,
        content_type: value.content_type,
        byte_size: value.byte_size,
        updated_at_millis: value.updated_at_millis,
        deleted_at_millis: value.deleted_at_millis,
        remote_version: value.remote_version,
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

fn conflict_title(local: &Value, cloud: &Value) -> String {
    [local, cloud]
        .into_iter()
        .find_map(|value| {
            value
                .get("title")
                .or_else(|| value.get("name"))
                .and_then(Value::as_str)
                .filter(|text| !text.trim().is_empty())
        })
        .unwrap_or("language")
        .to_owned()
}

fn conflict_value(key: &str, payload: &Value, snapshot: &AppSnapshot) -> SyncConflictValueView {
    let value = payload.get(key).cloned().unwrap_or(Value::Null);
    if value.is_null() {
        return SyncConflictValueView {
            kind: "empty".to_owned(),
            value,
            label: None,
        };
    }
    let (kind, display_value, label) = match key {
        "sort_index" => (
            "position",
            value
                .as_i64()
                .map(|position| Value::from(position + 1))
                .unwrap_or(value),
            None,
        ),
        "checklist_local_id" | "trashed_from_checklist_id" => {
            let id = value.as_str().unwrap_or_default();
            let label = snapshot
                .checklists
                .iter()
                .find(|list| list.id == id)
                .map(|list| list.name.clone());
            ("checklist", Value::String(id.to_owned()), label)
        }
        "completed" => (
            "status",
            Value::String(
                if value.as_bool().unwrap_or(false) {
                    "completed"
                } else {
                    "active"
                }
                .to_owned(),
            ),
            None,
        ),
        "priority" => (
            "priority",
            Value::String(value.as_str().unwrap_or_default().to_ascii_lowercase()),
            None,
        ),
        "reminder_repeat" => (
            "repeat",
            Value::String(value.as_str().unwrap_or_default().to_ascii_lowercase()),
            None,
        ),
        "language_mode" => ("language", value, None),
        "due_at_millis" | "trashed_at_millis" => ("timestamp", value, None),
        _ => ("text", value, None),
    };
    SyncConflictValueView {
        kind: kind.to_owned(),
        value: display_value,
        label,
    }
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
    fn independent_local_and_cloud_fields_merge_without_conflict() {
        let base = json!({ "title": "Base", "priority": "LOW", "completed": false });
        let local = json!({ "title": "Local", "priority": "LOW", "completed": false });
        let cloud = json!({ "title": "Base", "priority": "HIGH", "completed": false });

        let decision = three_way_merge("item", &base, &local, &cloud);

        assert!(decision.conflict_fields.is_empty());
        assert!(decision.needs_push);
        assert_eq!(decision.merged["title"], "Local");
        assert_eq!(decision.merged["priority"], "HIGH");
    }

    #[test]
    fn different_edits_to_the_same_field_require_review() {
        let base = json!({ "title": "Base" });
        let local = json!({ "title": "Local" });
        let cloud = json!({ "title": "Cloud" });

        let decision = three_way_merge("item", &base, &local, &cloud);

        assert_eq!(decision.conflict_fields, ["title"]);
        assert_eq!(decision.merged["title"], "Local");
    }

    #[test]
    fn identical_edits_do_not_require_review_or_another_push() {
        let base = json!({ "name": "Base", "sort_index": 0 });
        let local = json!({ "name": "Renamed", "sort_index": 0 });
        let cloud = json!({ "name": "Renamed", "sort_index": 0 });

        let decision = three_way_merge("checklist", &base, &local, &cloud);

        assert!(decision.conflict_fields.is_empty());
        assert!(!decision.needs_push);
    }

    #[test]
    fn synthetic_checklists_never_enter_cloud_sync() {
        assert!(is_synthetic_checklist_id(TRASH_CHECKLIST_ID));
        assert!(is_synthetic_checklist_id(SETTINGS_CHECKLIST_ID));
        assert!(!is_synthetic_checklist_id("main"));
    }

    #[test]
    fn mutation_request_preserves_the_durable_uuid() {
        let request = mutation_request("fixed-mutation-id", &MutationPayload::default());

        assert_eq!(request["p_mutation_uuid"], "fixed-mutation-id");
        assert_eq!(request["p_client_schema_version"], EXPECTED_SCHEMA);
    }

    #[test]
    fn cloud_todo_payload_never_contains_local_image_metadata() {
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
        let json = serde_json::to_value(remote).unwrap();
        assert!(json.get("image_local_name").is_none());
        assert!(json.get("image_remote_path").is_none());
        assert!(json.get("image_sync_state").is_none());
        for field in sync_fields("item") {
            assert!(
                json.get(*field).is_some(),
                "cloud conflict payload must contain {field}"
            );
        }
        assert!(json.get("dueAtMillis").is_none());
        assert!(json.get("reminderRepeat").is_none());
    }
}
