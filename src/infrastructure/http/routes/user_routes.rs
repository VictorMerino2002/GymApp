use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::users::{
        ChangeUserRoleUseCase, ChangeUserRoleUseCaseParams, ListUsersUseCase,
        ListUsersUseCaseParams,
    },
    domain::value_objects::Role,
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{AdminUser, ApiJson, ApiQuery},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct ListUsersQuery {
    username: String,
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn list_users(
    State(app_state): State<Arc<AppState>>,
    ApiQuery(query): ApiQuery<ListUsersQuery>,
) -> impl IntoResponse {
    let use_case = ListUsersUseCase {
        user_repository: app_state.user_repository.clone(),
    };

    let params = ListUsersUseCaseParams {
        username: query.username,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

#[derive(Deserialize)]
pub struct ChangeRoleRequest {
    role: Role,
}

pub async fn change_user_role(
    State(app_state): State<Arc<AppState>>,
    admin: AdminUser,
    Path(id): Path<String>,
    ApiJson(payload): ApiJson<ChangeRoleRequest>,
) -> impl IntoResponse {
    let use_case = ChangeUserRoleUseCase {
        user_repository: app_state.user_repository.clone(),
    };

    let params = ChangeUserRoleUseCaseParams {
        admin_id: admin.user_id,
        user_id: id,
        role: payload.role,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
