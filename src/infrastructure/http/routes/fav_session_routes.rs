use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::fav_sessions::{
        AddFavSessionUseCase, AddFavSessionUseCaseParams, ListFavSessionsUseCase,
        ListFavSessionsUseCaseParams, RemoveFavSessionUseCase, RemoveFavSessionUseCaseParams,
    },
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{ApiQuery, AuthUser},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct ListFavSessionsQuery {
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn add_fav_session(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let use_case = AddFavSessionUseCase {
        fav_session_repository: app_state.fav_session_repository.clone(),
        session_repository: app_state.session_repository.clone(),
    };

    let params = AddFavSessionUseCaseParams {
        user_id: user.user_id,
        session_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn remove_fav_session(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let use_case = RemoveFavSessionUseCase {
        fav_session_repository: app_state.fav_session_repository.clone(),
    };

    let params = RemoveFavSessionUseCaseParams {
        user_id: user.user_id,
        session_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn list_fav_sessions(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<ListFavSessionsQuery>,
) -> impl IntoResponse {
    let use_case = ListFavSessionsUseCase {
        fav_session_repository: app_state.fav_session_repository.clone(),
        session_repository: app_state.session_repository.clone(),
    };

    let params = ListFavSessionsUseCaseParams {
        user_id: user.user_id,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
