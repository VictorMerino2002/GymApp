use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{
    application::use_cases::auth::{
        GenerateAuthUseCaseResponse, LoginUseCase, LoginUseCaseParams, RegisterUseCase,
        RegisterUseCaseParams,
    },
    infrastructure::{
        bootstrap::AppState,
        http::{extractors::ApiJson, value_objects::ApiResponse},
    },
};

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    access: String,
    refresh: String,
}

impl From<GenerateAuthUseCaseResponse> for AuthResponse {
    fn from(response: GenerateAuthUseCaseResponse) -> Self {
        Self {
            access: response.access,
            refresh: response.refresh,
        }
    }
}

pub async fn login(
    State(app_state): State<Arc<AppState>>,
    ApiJson(payload): ApiJson<LoginRequest>,
) -> impl IntoResponse {
    let use_case = LoginUseCase {
        jwt_encoder: app_state.jwt_encoder.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = LoginUseCaseParams {
        username: payload.username,
    };

    let result = use_case.execute(params).await.map(AuthResponse::from);
    ApiResponse::default(result)
}

pub async fn register(
    State(app_state): State<Arc<AppState>>,
    ApiJson(payload): ApiJson<RegisterRequest>,
) -> impl IntoResponse {
    let use_case = RegisterUseCase {
        jwt_encoder: app_state.jwt_encoder.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = RegisterUseCaseParams {
        username: payload.username,
    };

    let result = use_case.execute(params).await.map(AuthResponse::from);
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}
