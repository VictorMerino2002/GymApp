use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};

use crate::{
    application::use_cases::auth::{
        GenerateAuthUseCaseResponse, LoginUseCase, LoginUseCaseParams, LogoutUseCase,
        LogoutUseCaseParams, RefreshTokenUseCase, RefreshTokenUseCaseParams, RegisterUseCase,
        RegisterUseCaseParams,
    },
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{ApiJson, AuthUser},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh: String,
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
        password_hasher: app_state.password_hasher.clone(),
    };

    let params = LoginUseCaseParams {
        username: payload.username,
        password: payload.password,
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
        password_hasher: app_state.password_hasher.clone(),
    };

    let params = RegisterUseCaseParams {
        username: payload.username,
        password: payload.password,
    };

    let result = use_case.execute(params).await.map(AuthResponse::from);
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}

pub async fn refresh(
    State(app_state): State<Arc<AppState>>,
    ApiJson(payload): ApiJson<RefreshRequest>,
) -> impl IntoResponse {
    let use_case = RefreshTokenUseCase {
        jwt_encoder: app_state.jwt_encoder.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = RefreshTokenUseCaseParams {
        refresh_token: payload.refresh,
    };

    let result = use_case.execute(params).await.map(AuthResponse::from);
    ApiResponse::default(result)
}

pub async fn logout(State(app_state): State<Arc<AppState>>, user: AuthUser) -> impl IntoResponse {
    let use_case = LogoutUseCase {
        user_repository: app_state.user_repository.clone(),
    };

    let params = LogoutUseCaseParams {
        user_id: user.user_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
