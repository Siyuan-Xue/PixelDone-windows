//! Local attachment and Supabase Storage boundary.

use std::{fs, path::Path};

use image::ImageFormat;
use sha2::{Digest, Sha256};

use crate::domain::AppError;

pub const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageMetadata {
    pub content_sha256: String,
    pub content_type: &'static str,
    pub extension: &'static str,
    pub byte_size: i64,
}

pub fn inspect_image_file(path: &Path) -> Result<(Vec<u8>, ImageMetadata), AppError> {
    let bytes = fs::read(path)?;
    let metadata = inspect_image_bytes(&bytes)?;
    Ok((bytes, metadata))
}

pub fn inspect_image_bytes(bytes: &[u8]) -> Result<ImageMetadata, AppError> {
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err(AppError::Validation(
            "Image must be no larger than 10 MiB".to_owned(),
        ));
    }
    let format = image::guess_format(bytes)
        .map_err(|_| AppError::Validation("Image must be JPEG, PNG, or WebP".to_owned()))?;
    let (content_type, extension) = match format {
        ImageFormat::Jpeg => ("image/jpeg", "jpg"),
        ImageFormat::Png => ("image/png", "png"),
        ImageFormat::WebP => ("image/webp", "webp"),
        _ => {
            return Err(AppError::Validation(
                "Image must be JPEG, PNG, or WebP".to_owned(),
            ));
        }
    };
    image::load_from_memory_with_format(bytes, format)
        .map_err(|_| AppError::Validation("The selected image is invalid".to_owned()))?;
    Ok(ImageMetadata {
        content_sha256: format!("{:x}", Sha256::digest(bytes)),
        content_type,
        extension,
        byte_size: bytes.len() as i64,
    })
}

pub fn cache_file_name(
    todo_id: &str,
    attachment_id: &str,
    hash: &str,
    content_type: &str,
) -> Result<String, AppError> {
    if !safe_identifier(todo_id) || !safe_identifier(attachment_id) || !is_sha256(hash) {
        return Err(AppError::Validation(
            "Invalid attachment metadata".to_owned(),
        ));
    }
    let extension = extension_for_content_type(content_type)?;
    Ok(format!(
        "{todo_id}-{attachment_id}-{}.{}",
        &hash[..16],
        extension
    ))
}

pub fn extension_for_content_type(value: &str) -> Result<&'static str, AppError> {
    match value {
        "image/jpeg" => Ok("jpg"),
        "image/png" => Ok("png"),
        "image/webp" => Ok("webp"),
        _ => Err(AppError::Validation("Unsupported image type".to_owned())),
    }
}

pub fn is_safe_object_path(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._-/".contains(&byte))
}

pub fn safe_file_name(value: &str) -> bool {
    !value.is_empty() && Path::new(value).file_name().and_then(|name| name.to_str()) == Some(value)
}

fn safe_identifier(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use base64::{Engine as _, engine::general_purpose::STANDARD};

    use super::*;

    #[test]
    fn validates_real_png_bytes_and_builds_a_safe_cache_name() {
        let bytes = STANDARD
            .decode("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .unwrap();
        let metadata = inspect_image_bytes(&bytes).unwrap();
        assert_eq!(metadata.content_type, "image/png");
        assert_eq!(metadata.byte_size, bytes.len() as i64);
        let name = cache_file_name(
            "todo-1",
            "2a4e8797-23ab-4afd-a95a-67a0f376fd6d",
            &metadata.content_sha256,
            metadata.content_type,
        )
        .unwrap();
        assert!(safe_file_name(&name));
        assert!(name.ends_with(".png"));
    }

    #[test]
    fn rejects_extension_only_or_path_traversal_inputs() {
        assert!(inspect_image_bytes(b"not a real image.png").is_err());
        assert!(!is_safe_object_path("owner/todo/../escape.png"));
        assert!(cache_file_name("../todo", "attachment", &"a".repeat(64), "image/png").is_err());
    }
}
