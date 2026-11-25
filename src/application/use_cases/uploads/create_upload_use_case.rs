use std::{collections::HashMap, sync::Arc, time::Duration};

use serde::Serialize;
use uuid::Uuid;

use crate::{
    application::ports::FileStoragePort,
    domain::value_objects::{Error, MediaKind, Role},
};

const UPLOAD_EXPIRATION: Duration = Duration::from_secs(15 * 60);
const MAX_IMAGE_BYTES: u64 = 5 * 1024 * 1024;
const MAX_VIDEO_BYTES: u64 = 100 * 1024 * 1024;
const IMAGE_TYPES: &[(&str, &str)] = &[
    ("image/jpeg", "jpg"),
    ("image/png", "png"),
    ("image/webp", "webp"),
];
const VIDEO_TYPES: &[(&str, &str)] = &[
    ("video/mp4", "mp4"),
    ("video/quicktime", "mov"),
    ("video/webm", "webm"),
];

#[derive(Debug, Serialize)]
pub struct UploadTicket {
    pub key: String,
    pub method: &'static str,
    pub upload_url: String,
    pub headers: HashMap<String, String>,
    pub file_url: String,
    pub expires_in_secs: u64,
}

pub struct CreateUploadUseCase {
    pub file_storage: Arc<dyn FileStoragePort>,
}

pub struct CreateUploadUseCaseParams {
    pub user_id: String,
    pub role: Role,
    pub kind: MediaKind,
    pub content_type: String,
    pub size_bytes: u64,
}

struct MediaRules {
    admin_only: bool,
    content_types: &'static [(&'static str, &'static str)],
    max_bytes: u64,
}

fn rules(kind: MediaKind) -> MediaRules {
    match kind {
        MediaKind::ExerciseImage => MediaRules {
            admin_only: true,
            content_types: IMAGE_TYPES,
            max_bytes: MAX_IMAGE_BYTES,
        },
        MediaKind::ExerciseVideo => MediaRules {
            admin_only: true,
            content_types: VIDEO_TYPES,
            max_bytes: MAX_VIDEO_BYTES,
        },
        MediaKind::ProfilePicture => MediaRules {
            admin_only: false,
            content_types: IMAGE_TYPES,
            max_bytes: MAX_IMAGE_BYTES,
        },
    }
}

fn key_prefix(kind: MediaKind, user_id: &str) -> String {
    match kind {
        MediaKind::ExerciseImage => "exercises/images".to_string(),
        MediaKind::ExerciseVideo => "exercises/videos".to_string(),
        MediaKind::ProfilePicture => format!("users/{user_id}"),
    }
}

impl CreateUploadUseCase {
    pub async fn execute(&self, params: CreateUploadUseCaseParams) -> Result<UploadTicket, Error> {
        let rules = rules(params.kind);

        if rules.admin_only && params.role != Role::Admin {
            return Err(Error::forbidden("Admin role required"));
        }

        let content_type = params.content_type.trim().to_lowercase();
        let extension = rules
            .content_types
            .iter()
            .find(|(allowed, _)| *allowed == content_type)
            .map(|(_, extension)| *extension)
            .ok_or_else(|| {
                let allowed: Vec<&str> = rules.content_types.iter().map(|(t, _)| *t).collect();
                Error::bad_request(&format!(
                    "content_type must be one of: {}",
                    allowed.join(", ")
                ))
            })?;

        if params.size_bytes == 0 || params.size_bytes > rules.max_bytes {
            return Err(Error::bad_request(&format!(
                "size_bytes must be between 1 and {}",
                rules.max_bytes
            )));
        }

        let key = format!(
            "{}/{}.{extension}",
            key_prefix(params.kind, &params.user_id),
            Uuid::new_v4()
        );

        let presigned = self
            .file_storage
            .presign_upload(&key, &content_type, params.size_bytes, UPLOAD_EXPIRATION)
            .await?;

        Ok(UploadTicket {
            file_url: self.file_storage.public_url(&key),
            key,
            method: "PUT",
            upload_url: presigned.url,
            headers: presigned.headers.into_iter().collect(),
            expires_in_secs: UPLOAD_EXPIRATION.as_secs(),
        })
    }
}
