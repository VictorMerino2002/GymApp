use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::{
    application::use_cases::exercise_history::{
        CreateExerciseHistoryUseCase, CreateExerciseHistoryUseCaseParams,
        DeleteExerciseHistoryUseCase, DeleteExerciseHistoryUseCaseParams,
        GetExerciseHistoryUseCase, GetExerciseHistoryUseCaseParams, ListExerciseHistoryUseCase,
        ListExerciseHistoryUseCaseParams,
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
pub struct ExerciseHistoryRequest {
    exercise_id: String,
    session_id: Option<String>,
    performed_at: Option<DateTime<Utc>>,
    weight_kg: Option<f32>,
    reps: Option<u16>,
}

#[derive(Deserialize)]
pub struct ListExerciseHistoryQuery {
    exercise_id: Option<String>,
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn create_exercise_history(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiJson(payload): ApiJson<ExerciseHistoryRequest>,
) -> impl IntoResponse {
    let use_case = CreateExerciseHistoryUseCase {
        exercise_history_repository: app_state.exercise_history_repository.clone(),
        exercise_repository: app_state.exercise_repository.clone(),
        session_repository: app_state.session_repository.clone(),
        user_repository: app_state.user_repository.clone(),
        activity_repository: app_state.activity_repository.clone(),
    };

    let params = CreateExerciseHistoryUseCaseParams {
        user_id: user.user_id,
        exercise_id: payload.exercise_id,
        session_id: payload.session_id,
        performed_at: payload.performed_at,
        weight_kg: payload.weight_kg,
        reps: payload.reps,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}

pub async fn list_exercise_history(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<ListExerciseHistoryQuery>,
) -> impl IntoResponse {
    let use_case = ListExerciseHistoryUseCase {
        exercise_history_repository: app_state.exercise_history_repository.clone(),
    };

    let params = ListExerciseHistoryUseCaseParams {
        user_id: user.user_id,
        exercise_id: query.exercise_id,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn get_exercise_history(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = GetExerciseHistoryUseCase {
        exercise_history_repository: app_state.exercise_history_repository.clone(),
    };

    let params = GetExerciseHistoryUseCaseParams {
        user_id: user.user_id,
        id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn delete_exercise_history(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = DeleteExerciseHistoryUseCase {
        exercise_history_repository: app_state.exercise_history_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = DeleteExerciseHistoryUseCaseParams {
        user_id: user.user_id,
        id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
