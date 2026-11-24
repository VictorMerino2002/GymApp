use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use serde::{Deserialize, Deserializer};

use crate::{
    application::use_cases::me::{
        ChangePasswordUseCase, ChangePasswordUseCaseParams, GetProfileUseCase,
        GetProfileUseCaseParams, UpdateProfileUseCase, UpdateProfileUseCaseParams,
    },
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{ApiJson, AuthUser},
            routes::auth_routes::AuthResponse,
            value_objects::ApiResponse,
        },
    },
};

fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    username: Option<String>,
    #[serde(default, deserialize_with = "present")]
    picture_url: Option<Option<String>>,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

pub async fn get_profile(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
) -> impl IntoResponse {
    let use_case = GetProfileUseCase {
        user_repository: app_state.user_repository.clone(),
    };

    let params = GetProfileUseCaseParams {
        user_id: user.user_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn update_profile(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiJson(payload): ApiJson<UpdateProfileRequest>,
) -> impl IntoResponse {
    let use_case = UpdateProfileUseCase {
        user_repository: app_state.user_repository.clone(),
    };

    let params = UpdateProfileUseCaseParams {
        user_id: user.user_id,
        username: payload.username,
        picture_url: payload.picture_url,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn change_password(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiJson(payload): ApiJson<ChangePasswordRequest>,
) -> impl IntoResponse {
    let use_case = ChangePasswordUseCase {
        user_repository: app_state.user_repository.clone(),
        password_hasher: app_state.password_hasher.clone(),
        jwt_encoder: app_state.jwt_encoder.clone(),
    };

    let params = ChangePasswordUseCaseParams {
        user_id: user.user_id,
        current_password: payload.current_password,
        new_password: payload.new_password,
    };

    let result = use_case.execute(params).await.map(AuthResponse::from);
    ApiResponse::default(result)
}
