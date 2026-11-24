use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use serde::Deserialize;

use crate::{
    application::use_cases::users::{ListUsersUseCase, ListUsersUseCaseParams},
    infrastructure::{
        bootstrap::AppState,
        http::{extractors::ApiQuery, value_objects::ApiResponse},
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
