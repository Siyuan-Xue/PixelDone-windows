use std::{fs, io::Cursor};

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{GenericImageView, ImageFormat, ImageReader};
use tauri::State;
use uuid::Uuid;

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, unix_now_millis},
    infrastructure::{
        db::LocalTodoAttachment,
        image::{inspect_image_bytes, inspect_image_file, safe_file_name},
    },
};

const MAX_PREVIEW_DIMENSION: u32 = 2048;

#[tauri::command]
pub async fn attach_todo_image(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    todo_id: String,
    source_path: String,
) -> Result<MutationResult, AppError> {
    let source = fs::canonicalize(&source_path)
        .map_err(|error| AppError::Validation(format!("Unable to read image: {error}")))?;
    if !fs::metadata(&source)?.is_file() {
        return Err(AppError::Validation(
            "The selected path is not a file".to_owned(),
        ));
    }
    let (bytes, image) = inspect_image_file(&source)?;
    let file_name = format!("{}-{}.{}", todo_id, Uuid::new_v4(), image.extension);
    let destination = state.paths.attachments.join(&file_name);
    fs::write(&destination, &bytes)?;

    let (previous_file, previous_attachment) = {
        let runtime = state.inner.lock().await;
        let previous_file = runtime
            .snapshot
            .checklists
            .iter()
            .find(|list| list.id == checklist_id)
            .and_then(|list| list.items.iter().find(|item| item.id == todo_id))
            .and_then(|item| item.image_file_name.clone());
        let previous_attachment = runtime.repository.attachment(&todo_id).await?;
        (previous_file, previous_attachment)
    };
    let updated_at_millis = unix_now_millis();
    let previous_object_path = previous_attachment
        .as_ref()
        .and_then(|value| value.object_path.clone());
    let result = mutate(state.clone(), expected_revision, |snapshot| {
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        let item = checklist
            .items
            .iter_mut()
            .find(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        item.image_file_name = Some(file_name.clone());
        item.updated_at_millis = updated_at_millis;
        Ok(vec![checklist_id, todo_id.clone()])
    })
    .await;

    if result.is_err() {
        let _ = fs::remove_file(destination);
        return result;
    }

    let repository = state.inner.lock().await.repository.clone();
    repository
        .save_attachment(&LocalTodoAttachment {
            todo_id: todo_id.clone(),
            local_file_name: Some(file_name),
            attachment_id: previous_attachment
                .as_ref()
                .and_then(|value| value.attachment_id.clone())
                .or_else(|| Some(Uuid::new_v4().to_string())),
            object_path: None,
            content_sha256: Some(image.content_sha256),
            content_type: Some(image.content_type.to_owned()),
            byte_size: Some(image.byte_size),
            updated_at_millis,
            deleted_at_millis: None,
            remote_version: previous_attachment.and_then(|value| value.remote_version),
            sync_state: "PENDING_UPLOAD".to_owned(),
            last_error: None,
        })
        .await?;
    if let Some(path) = previous_object_path {
        repository
            .queue_local_image_cleanup(&todo_id, &path)
            .await?;
    }
    state.sync_notify.notify_one();
    if let Some(previous) = previous_file
        && safe_file_name(&previous)
    {
        let _ = fs::remove_file(state.paths.attachments.join(previous));
    }
    result
}

#[tauri::command]
pub async fn delete_todo_image(
    state: State<'_, ManagedAppState>,
    expected_revision: i64,
    checklist_id: String,
    todo_id: String,
) -> Result<MutationResult, AppError> {
    let (previous_file, previous_attachment) = {
        let runtime = state.inner.lock().await;
        let file = runtime
            .snapshot
            .checklists
            .iter()
            .find(|list| list.id == checklist_id)
            .and_then(|list| list.items.iter().find(|item| item.id == todo_id))
            .and_then(|item| item.image_file_name.clone());
        let attachment = runtime.repository.attachment(&todo_id).await?;
        (file, attachment)
    };
    let updated_at_millis = unix_now_millis();
    let previous_object_path = previous_attachment
        .as_ref()
        .and_then(|value| value.object_path.clone());
    let result = mutate(state.clone(), expected_revision, |snapshot| {
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        let item = checklist
            .items
            .iter_mut()
            .find(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        item.image_file_name = None;
        item.updated_at_millis = updated_at_millis;
        Ok(vec![checklist_id, todo_id.clone()])
    })
    .await?;

    let repository = state.inner.lock().await.repository.clone();
    repository
        .save_attachment(&LocalTodoAttachment {
            todo_id: todo_id.clone(),
            updated_at_millis,
            deleted_at_millis: Some(updated_at_millis),
            remote_version: previous_attachment.and_then(|value| value.remote_version),
            sync_state: "METADATA_PENDING".to_owned(),
            ..LocalTodoAttachment::default()
        })
        .await?;
    if let Some(path) = previous_object_path {
        repository
            .queue_local_image_cleanup(&todo_id, &path)
            .await?;
    }
    state.sync_notify.notify_one();
    if let Some(previous) = previous_file
        && safe_file_name(&previous)
    {
        let _ = fs::remove_file(state.paths.attachments.join(previous));
    }
    Ok(result)
}

#[tauri::command]
pub async fn load_todo_image_preview(
    state: State<'_, ManagedAppState>,
    todo_id: String,
) -> Result<String, AppError> {
    let (file_name, attachment, repository) = {
        let runtime = state.inner.lock().await;
        let item_file = runtime
            .snapshot
            .checklists
            .iter()
            .flat_map(|list| list.items.iter())
            .find(|item| item.id == todo_id)
            .and_then(|item| item.image_file_name.clone());
        let attachment = runtime.repository.attachment(&todo_id).await?;
        let file_name = item_file
            .or_else(|| {
                attachment
                    .as_ref()
                    .and_then(|value| value.local_file_name.clone())
            })
            .ok_or_else(|| AppError::NotFound(format!("image for {todo_id}")))?;
        (file_name, attachment, runtime.repository.clone())
    };
    if !safe_file_name(&file_name) {
        return Err(AppError::Validation("Invalid image path".to_owned()));
    }
    let path = state.paths.attachments.join(&file_name);
    if !path.is_file() {
        let attachment = attachment
            .ok_or_else(|| AppError::NotFound(format!("image metadata for {todo_id}")))?;
        let object_path = attachment
            .object_path
            .as_deref()
            .ok_or_else(|| AppError::NotFound(format!("remote image for {todo_id}")))?;
        let expected_hash = attachment
            .content_sha256
            .as_deref()
            .ok_or_else(|| AppError::Validation("Image hash is missing".to_owned()))?;
        let expected_type = attachment
            .content_type
            .as_deref()
            .ok_or_else(|| AppError::Validation("Image type is missing".to_owned()))?;
        let expected_size = attachment
            .byte_size
            .ok_or_else(|| AppError::Validation("Image size is missing".to_owned()))?;
        let session = state
            .session
            .lock()
            .await
            .clone()
            .ok_or_else(|| AppError::Auth("Sign in to download this image".to_owned()))?;
        let mut session = state.cloud.refresh_if_needed(&session, false).await?;
        state.credentials.save(&session)?;
        *state.session.lock().await = Some(session.clone());
        let bytes = match state.cloud.download_todo_image(&session, object_path).await {
            Ok(bytes) => bytes,
            Err(_) => {
                session = state.cloud.refresh_if_needed(&session, true).await?;
                state.credentials.save(&session)?;
                *state.session.lock().await = Some(session.clone());
                state
                    .cloud
                    .download_todo_image(&session, object_path)
                    .await?
            }
        };
        let downloaded = inspect_image_bytes(&bytes)?;
        if downloaded.content_sha256 != expected_hash
            || downloaded.content_type != expected_type
            || downloaded.byte_size != expected_size
        {
            return Err(AppError::Validation(
                "Downloaded image failed integrity validation".to_owned(),
            ));
        }
        let temporary = path.with_extension("download");
        fs::write(&temporary, bytes)?;
        fs::rename(&temporary, &path)?;
        repository
            .save_attachment(&LocalTodoAttachment {
                local_file_name: Some(file_name.clone()),
                sync_state: "SYNCED".to_owned(),
                last_error: None,
                ..attachment
            })
            .await?;
    }

    let reader = ImageReader::open(&path)
        .map_err(|error| AppError::Validation(error.to_string()))?
        .with_guessed_format()
        .map_err(|error| AppError::Validation(error.to_string()))?;
    let image = reader
        .decode()
        .map_err(|error| AppError::Validation(error.to_string()))?;
    let (width, height) = image.dimensions();
    let preview = if width > MAX_PREVIEW_DIMENSION || height > MAX_PREVIEW_DIMENSION {
        image.thumbnail(MAX_PREVIEW_DIMENSION, MAX_PREVIEW_DIMENSION)
    } else {
        image
    };
    let mut output = Cursor::new(Vec::new());
    preview
        .write_to(&mut output, ImageFormat::Png)
        .map_err(|error| AppError::Validation(error.to_string()))?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(output.into_inner())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attachment_names_cannot_escape_the_attachment_directory() {
        assert!(safe_file_name("todo-1.png"));
        assert!(!safe_file_name("../todo-1.png"));
        assert!(!safe_file_name("folder/todo-1.png"));
    }
}
