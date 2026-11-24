use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::friendships::{
        AcceptFriendRequestUseCase, AcceptFriendRequestUseCaseParams, ListFriendsUseCase,
        ListFriendsUseCaseParams, RemoveFriendshipUseCase, RemoveFriendshipUseCaseParams,
        SendFriendRequestUseCase, SendFriendRequestUseCaseParams,
    },
    domain::entities::FriendshipStatus,
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{ApiQuery, AuthUser},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct ListFriendsQuery {
    status: Option<FriendshipStatus>,
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn send_friend_request(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(friend_id): Path<String>,
) -> impl IntoResponse {
    let use_case = SendFriendRequestUseCase {
        friendship_repository: app_state.friendship_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = SendFriendRequestUseCaseParams {
        user_id: user.user_id,
        friend_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}

pub async fn accept_friend_request(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(friend_id): Path<String>,
) -> impl IntoResponse {
    let use_case = AcceptFriendRequestUseCase {
        friendship_repository: app_state.friendship_repository.clone(),
    };

    let params = AcceptFriendRequestUseCaseParams {
        user_id: user.user_id,
        friend_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn remove_friendship(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(friend_id): Path<String>,
) -> impl IntoResponse {
    let use_case = RemoveFriendshipUseCase {
        friendship_repository: app_state.friendship_repository.clone(),
    };

    let params = RemoveFriendshipUseCaseParams {
        user_id: user.user_id,
        friend_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn list_friends(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<ListFriendsQuery>,
) -> impl IntoResponse {
    let use_case = ListFriendsUseCase {
        friendship_repository: app_state.friendship_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = ListFriendsUseCaseParams {
        user_id: user.user_id,
        status: query.status,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
