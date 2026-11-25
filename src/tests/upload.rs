use std::{collections::HashMap, sync::Arc, time::Duration};

use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use axum::http::StatusCode;

use crate::{
    application::{
        ports::FileStoragePort,
        use_cases::uploads::{CreateUploadUseCase, CreateUploadUseCaseParams, UploadTicket},
    },
    domain::value_objects::{Error, MediaKind, Role},
    infrastructure::adapters::S3FileStorage,
    tests::support::FakeFileStorage,
};

const MIB: u64 = 1024 * 1024;

async fn upload(
    storage: &Arc<FakeFileStorage>,
    role: Role,
    kind: MediaKind,
    content_type: &str,
    size_bytes: u64,
) -> Result<UploadTicket, Error> {
    CreateUploadUseCase {
        file_storage: storage.clone(),
    }
    .execute(CreateUploadUseCaseParams {
        user_id: "u1".to_string(),
        role,
        kind,
        content_type: content_type.to_string(),
        size_bytes,
    })
    .await
}

#[tokio::test]
async fn profile_picture_upload_returns_presigned_ticket() {
    let storage = Arc::new(FakeFileStorage::default());
    let ticket = upload(
        &storage,
        Role::User,
        MediaKind::ProfilePicture,
        "image/png",
        2 * MIB,
    )
    .await
    .unwrap();

    assert!(ticket.key.starts_with("users/u1/"));
    assert!(ticket.key.ends_with(".png"));
    assert_eq!(ticket.method, "PUT");
    assert_eq!(
        ticket.file_url,
        format!("https://cdn.example.com/{}", ticket.key)
    );
    assert!(ticket.upload_url.contains(&ticket.key));
    assert_eq!(
        ticket.headers.get("content-type").map(String::as_str),
        Some("image/png")
    );
    assert_eq!(ticket.expires_in_secs, 900);

    let calls = storage.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].key, ticket.key);
    assert_eq!(calls[0].content_type, "image/png");
    assert_eq!(calls[0].content_length, 2 * MIB);
    assert_eq!(calls[0].expires_in, Duration::from_secs(900));
}

#[tokio::test]
async fn every_upload_gets_a_unique_key() {
    let storage = Arc::new(FakeFileStorage::default());
    let first = upload(
        &storage,
        Role::User,
        MediaKind::ProfilePicture,
        "image/jpeg",
        MIB,
    )
    .await
    .unwrap();
    let second = upload(
        &storage,
        Role::User,
        MediaKind::ProfilePicture,
        "image/jpeg",
        MIB,
    )
    .await
    .unwrap();

    assert_ne!(first.key, second.key);
    assert!(first.key.ends_with(".jpg"));
}

#[tokio::test]
async fn content_type_is_normalized() {
    let storage = Arc::new(FakeFileStorage::default());
    let ticket = upload(
        &storage,
        Role::User,
        MediaKind::ProfilePicture,
        " Image/WEBP ",
        MIB,
    )
    .await
    .unwrap();

    assert!(ticket.key.ends_with(".webp"));
    assert_eq!(storage.calls.lock().unwrap()[0].content_type, "image/webp");
}

#[tokio::test]
async fn admins_can_upload_exercise_media() {
    let storage = Arc::new(FakeFileStorage::default());
    let image = upload(
        &storage,
        Role::Admin,
        MediaKind::ExerciseImage,
        "image/jpeg",
        MIB,
    )
    .await
    .unwrap();
    let video = upload(
        &storage,
        Role::Admin,
        MediaKind::ExerciseVideo,
        "video/quicktime",
        80 * MIB,
    )
    .await
    .unwrap();

    assert!(image.key.starts_with("exercises/images/"));
    assert!(video.key.starts_with("exercises/videos/"));
    assert!(video.key.ends_with(".mov"));
}

#[tokio::test]
async fn users_cannot_upload_exercise_media() {
    let storage = Arc::new(FakeFileStorage::default());

    for (kind, content_type) in [
        (MediaKind::ExerciseImage, "image/png"),
        (MediaKind::ExerciseVideo, "video/mp4"),
    ] {
        let error = upload(&storage, Role::User, kind, content_type, MIB)
            .await
            .unwrap_err();
        assert_eq!(error.code, StatusCode::FORBIDDEN);
    }
    assert!(storage.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn rejects_content_types_not_allowed_for_the_kind() {
    let storage = Arc::new(FakeFileStorage::default());
    let invalid = [
        (MediaKind::ProfilePicture, "video/mp4"),
        (MediaKind::ProfilePicture, "image/gif"),
        (MediaKind::ExerciseImage, "application/pdf"),
        (MediaKind::ExerciseVideo, "image/png"),
    ];

    for (kind, content_type) in invalid {
        let error = upload(&storage, Role::Admin, kind, content_type, MIB)
            .await
            .unwrap_err();
        assert_eq!(error.code, StatusCode::BAD_REQUEST, "{content_type}");
    }
    assert!(storage.calls.lock().unwrap().is_empty());
}

#[tokio::test]
async fn enforces_size_limits() {
    let storage = Arc::new(FakeFileStorage::default());
    let cases = [
        (MediaKind::ProfilePicture, "image/png", 0, false),
        (MediaKind::ProfilePicture, "image/png", 5 * MIB, true),
        (MediaKind::ProfilePicture, "image/png", 5 * MIB + 1, false),
        (MediaKind::ExerciseVideo, "video/mp4", 100 * MIB, true),
        (MediaKind::ExerciseVideo, "video/mp4", 100 * MIB + 1, false),
    ];

    for (kind, content_type, size, allowed) in cases {
        let result = upload(&storage, Role::Admin, kind, content_type, size).await;
        match allowed {
            true => assert!(result.is_ok(), "{size} should be allowed"),
            false => assert_eq!(result.unwrap_err().code, StatusCode::BAD_REQUEST),
        }
    }
}

fn s3_storage() -> S3FileStorage {
    let config = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("eu-west-1"))
        .credentials_provider(Credentials::new("AKIDTEST", "secret", None, None, "test"))
        .build();

    S3FileStorage::new(
        aws_sdk_s3::Client::from_conf(config),
        "gym-media",
        "https://cdn.example.com/",
    )
}

#[tokio::test]
async fn s3_presigned_upload_signs_content_type_and_length() {
    let presigned = s3_storage()
        .presign_upload(
            "users/u1/pic.png",
            "image/png",
            1234,
            Duration::from_secs(900),
        )
        .await
        .unwrap();

    let headers: HashMap<String, String> = presigned.headers.into_iter().collect();
    assert!(presigned.url.contains("gym-media"));
    assert!(presigned.url.contains("users/u1/pic.png"));
    assert!(presigned.url.contains("X-Amz-Expires=900"));
    assert!(
        presigned
            .url
            .contains("X-Amz-SignedHeaders=content-length%3Bcontent-type%3Bhost")
    );
    assert_eq!(
        headers.get("content-type").map(String::as_str),
        Some("image/png")
    );
    assert_eq!(
        headers.get("content-length").map(String::as_str),
        Some("1234")
    );
}

#[test]
fn s3_public_url_uses_base_url_without_double_slash() {
    assert_eq!(
        s3_storage().public_url("users/u1/pic.png"),
        "https://cdn.example.com/users/u1/pic.png"
    );
}
