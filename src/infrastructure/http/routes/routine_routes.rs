use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::routines::{
        CreateRoutineUseCase, CreateRoutineUseCaseParams, DeleteRoutineUseCase,
        DeleteRoutineUseCaseParams, GetRoutineUseCase, GetRoutineUseCaseParams,
        ListRoutinesUseCase, ListRoutinesUseCaseParams, RoutineInput, UpdateRoutineUseCase,
        UpdateRoutineUseCaseParams,
    },
    domain::value_objects::DifficultyLevel,
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{AdminUser, ApiJson, ApiQuery},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct RoutineRequest {
    title: String,
    level: DifficultyLevel,
    duration_weeks: u16,
    #[serde(default)]
    description: String,
    session_ids: Vec<String>,
}

impl From<RoutineRequest> for RoutineInput {
    fn from(request: RoutineRequest) -> Self {
        Self {
            title: request.title,
            level: request.level,
            duration_weeks: request.duration_weeks,
            description: request.description,
            session_ids: request.session_ids,
        }
    }
}

#[derive(Deserialize)]
pub struct ListRoutinesQuery {
    author_id: Option<String>,
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn create_routine(
    State(app_state): State<Arc<AppState>>,
    admin: AdminUser,
    ApiJson(payload): ApiJson<RoutineRequest>,
) -> impl IntoResponse {
    let use_case = CreateRoutineUseCase {
        routine_repository: app_state.routine_repository.clone(),
        session_repository: app_state.session_repository.clone(),
        user_repository: app_state.user_repository.clone(),
    };

    let params = CreateRoutineUseCaseParams {
        user_id: admin.user_id,
        routine: payload.into(),
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}

pub async fn list_routines(
    State(app_state): State<Arc<AppState>>,
    ApiQuery(query): ApiQuery<ListRoutinesQuery>,
) -> impl IntoResponse {
    let use_case = ListRoutinesUseCase {
        routine_repository: app_state.routine_repository.clone(),
    };

    let params = ListRoutinesUseCaseParams {
        author_id: query.author_id,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn get_routine(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = GetRoutineUseCase {
        routine_repository: app_state.routine_repository.clone(),
    };

    let params = GetRoutineUseCaseParams { id };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn update_routine(
    State(app_state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(id): Path<String>,
    ApiJson(payload): ApiJson<RoutineRequest>,
) -> impl IntoResponse {
    let use_case = UpdateRoutineUseCase {
        routine_repository: app_state.routine_repository.clone(),
        session_repository: app_state.session_repository.clone(),
    };

    let params = UpdateRoutineUseCaseParams {
        id,
        routine: payload.into(),
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn delete_routine(
    State(app_state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = DeleteRoutineUseCase {
        routine_repository: app_state.routine_repository.clone(),
    };

    let params = DeleteRoutineUseCaseParams { id };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
