use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::sessions::{
        CreateSessionUseCase, CreateSessionUseCaseParams, DeleteSessionUseCase,
        DeleteSessionUseCaseParams, GetSessionUseCase, GetSessionUseCaseParams,
        ListSessionsUseCase, ListSessionsUseCaseParams, SessionExerciseInput, UpdateSessionUseCase,
        UpdateSessionUseCaseParams,
    },
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{ApiJson, ApiQuery, AuthUser},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct SessionExerciseRequest {
    exercise_id: String,
    sets: u16,
    reps: u16,
    rest_secs: u16,
}

impl From<SessionExerciseRequest> for SessionExerciseInput {
    fn from(request: SessionExerciseRequest) -> Self {
        Self {
            exercise_id: request.exercise_id,
            sets: request.sets,
            reps: request.reps,
            rest_secs: request.rest_secs,
        }
    }
}

#[derive(Deserialize)]
pub struct SessionRequest {
    name: String,
    exercises: Vec<SessionExerciseRequest>,
}

#[derive(Deserialize)]
pub struct ListSessionsQuery {
    author_id: Option<String>,
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn create_session(
    State(app_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    ApiJson(payload): ApiJson<SessionRequest>,
) -> impl IntoResponse {
    let use_case = CreateSessionUseCase {
        session_repository: app_state.session_repository.clone(),
        exercise_repository: app_state.exercise_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = CreateSessionUseCaseParams {
        user_id: auth_user.user_id,
        name: payload.name,
        exercises: payload.exercises.into_iter().map(Into::into).collect(),
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}

pub async fn list_sessions(
    State(app_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    ApiQuery(query): ApiQuery<ListSessionsQuery>,
) -> impl IntoResponse {
    let use_case = ListSessionsUseCase {
        session_repository: app_state.session_repository.clone(),
    };

    let params = ListSessionsUseCaseParams {
        author_id: query.author_id.unwrap_or(auth_user.user_id),
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn get_session(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = GetSessionUseCase {
        session_repository: app_state.session_repository.clone(),
    };

    let params = GetSessionUseCaseParams { id };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn update_session(
    State(app_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<String>,
    ApiJson(payload): ApiJson<SessionRequest>,
) -> impl IntoResponse {
    let use_case = UpdateSessionUseCase {
        session_repository: app_state.session_repository.clone(),
        exercise_repository: app_state.exercise_repository.clone(),
    };

    let params = UpdateSessionUseCaseParams {
        user_id: auth_user.user_id,
        id,
        name: payload.name,
        exercises: payload.exercises.into_iter().map(Into::into).collect(),
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn delete_session(
    State(app_state): State<Arc<AppState>>,
    auth_user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = DeleteSessionUseCase {
        session_repository: app_state.session_repository.clone(),
    };

    let params = DeleteSessionUseCaseParams {
        user_id: auth_user.user_id,
        id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
