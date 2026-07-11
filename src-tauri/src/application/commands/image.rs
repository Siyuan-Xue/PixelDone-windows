use std::{fs, io::Cursor, path::Path};

use base64::{Engine, engine::general_purpose::STANDARD};
use image::{GenericImageView, ImageFormat, ImageReader};
use tauri::State;
use uuid::Uuid;

use crate::{
    application::{commands::mutate, state::ManagedAppState},
    domain::{AppError, MutationResult, unix_now_millis},
};

const MAX_IMAGE_BYTES: u64 = 50 * 1024 * 1024;
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
        .map_err(|error| AppError::Validation(format!("无法读取图片：{error}")))?;
    let metadata = fs::metadata(&source)?;
    if !metadata.is_file() || metadata.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Validation("图片必须小于 50 MB".to_owned()));
    }
    let extension = supported_extension(&source)?;
    let file_name = format!("{}-{}.{}", todo_id, Uuid::new_v4(), extension);
    let destination = state.paths.attachments.join(&file_name);
    fs::copy(&source, &destination)?;

    let previous_file = {
        let runtime = state.inner.lock().await;
        runtime
            .snapshot
            .checklists
            .iter()
            .find(|list| list.id == checklist_id)
            .and_then(|list| list.items.iter().find(|item| item.id == todo_id))
            .and_then(|item| item.image_file_name.clone())
    };
    let result = mutate(state.clone(), expected_revision, |snapshot| {
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        let item = checklist
            .items
            .iter_mut()
            .find(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        item.image_file_name = Some(file_name.clone());
        item.updated_at_millis = unix_now_millis();
        Ok(vec![checklist_id, todo_id])
    })
    .await;
    if result.is_err() {
        let _ = fs::remove_file(destination);
    } else if let Some(previous) = previous_file
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
    let previous_file = {
        let runtime = state.inner.lock().await;
        runtime
            .snapshot
            .checklists
            .iter()
            .find(|list| list.id == checklist_id)
            .and_then(|list| list.items.iter().find(|item| item.id == todo_id))
            .and_then(|item| item.image_file_name.clone())
    };
    let result = mutate(state.clone(), expected_revision, |snapshot| {
        let checklist = snapshot.checklist_mut(&checklist_id)?;
        let item = checklist
            .items
            .iter_mut()
            .find(|item| item.id == todo_id)
            .ok_or_else(|| AppError::NotFound(format!("todo {todo_id}")))?;
        item.image_file_name = None;
        item.updated_at_millis = unix_now_millis();
        Ok(vec![checklist_id, todo_id])
    })
    .await?;
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
    let file_name = {
        let runtime = state.inner.lock().await;
        runtime
            .snapshot
            .checklists
            .iter()
            .flat_map(|list| list.items.iter())
            .find(|item| item.id == todo_id)
            .and_then(|item| item.image_file_name.clone())
            .ok_or_else(|| AppError::NotFound(format!("image for {todo_id}")))?
    };
    if !safe_file_name(&file_name) {
        return Err(AppError::Validation("图片路径无效".to_owned()));
    }
    let path = state.paths.attachments.join(file_name);
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

fn supported_extension(path: &Path) -> Result<&'static str, AppError> {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => Ok("jpg"),
        "png" => Ok("png"),
        "webp" => Ok("webp"),
        _ => Err(AppError::Validation(
            "仅支持 JPEG、PNG 和 WebP 图片".to_owned(),
        )),
    }
}

fn safe_file_name(value: &str) -> bool {
    !value.is_empty() && Path::new(value).file_name().and_then(|name| name.to_str()) == Some(value)
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
