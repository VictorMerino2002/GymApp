use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::social::{
        GetFeedUseCase, GetFeedUseCaseParams, GetLeaderboardUseCase, GetLeaderboardUseCaseParams,
        GetUserProfileUseCase, GetUserProfileUseCaseParams,
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
pub struct FeedQuery {
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn get_user_profile(
    State(app_state): State<Arc<AppState>>,
    viewer: AuthUser,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let use_case = GetUserProfileUseCase {
        user_repository: app_state.user_repository.clone(),
        friendship_repository: app_state.friendship_repository.clone(),
    };

    let params = GetUserProfileUseCaseParams {
        viewer_id: viewer.user_id,
        user_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn get_feed(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<FeedQuery>,
) -> impl IntoResponse {
    let use_case = GetFeedUseCase {
        activity_repository: app_state.activity_repository.clone(),
        friendship_repository: app_state.friendship_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = GetFeedUseCaseParams {
        user_id: user.user_id,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn get_leaderboard(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
) -> impl IntoResponse {
    let use_case = GetLeaderboardUseCase {
        friendship_repository: app_state.friendship_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = GetLeaderboardUseCaseParams {
        user_id: user.user_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
