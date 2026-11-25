use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde::Deserialize;

use crate::{
    application::use_cases::uploads::{CreateUploadUseCase, CreateUploadUseCaseParams},
    domain::value_objects::MediaKind,
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{ApiJson, AuthUser},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct CreateUploadRequest {
    kind: MediaKind,
    content_type: String,
    size_bytes: u64,
}

pub async fn create_upload(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiJson(payload): ApiJson<CreateUploadRequest>,
) -> impl IntoResponse {
    let use_case = CreateUploadUseCase {
        file_storage: app_state.file_storage.clone(),
    };

    let params = CreateUploadUseCaseParams {
        user_id: user.user_id,
        role: user.role,
        kind: payload.kind,
        content_type: payload.content_type,
        size_bytes: payload.size_bytes,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}
