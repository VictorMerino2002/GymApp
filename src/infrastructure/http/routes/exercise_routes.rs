use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::exercises::{
        CreateExerciseUseCase, CreateExerciseUseCaseParams, DeleteExerciseUseCase,
        DeleteExerciseUseCaseParams, ExerciseInput, GetExerciseUseCase, GetExerciseUseCaseParams,
        ListExercisesUseCase, ListExercisesUseCaseParams, UpdateExerciseUseCase,
        UpdateExerciseUseCaseParams,
    },
    domain::value_objects::{ExerciseCategory, Muscle},
    infrastructure::{
        bootstrap::AppState,
        http::{
            extractors::{AdminUser, ApiJson, ApiQuery},
            value_objects::ApiResponse,
        },
    },
};

#[derive(Deserialize)]
pub struct ExerciseRequest {
    name: String,
    #[serde(default)]
    category: ExerciseCategory,
    primary_muscles: Vec<Muscle>,
    #[serde(default)]
    secondary_muscles: Vec<Muscle>,
    #[serde(default)]
    description: String,
    #[serde(default)]
    instructions: Vec<String>,
    video_url: Option<String>,
    img_url: Option<String>,
}

impl From<ExerciseRequest> for ExerciseInput {
    fn from(request: ExerciseRequest) -> Self {
        Self {
            name: request.name,
            category: request.category,
            primary_muscles: request.primary_muscles,
            secondary_muscles: request.secondary_muscles,
            description: request.description,
            instructions: request.instructions,
            video_url: request.video_url,
            img_url: request.img_url,
        }
    }
}

#[derive(Deserialize)]
pub struct ListExercisesQuery {
    limit: Option<u16>,
    cursor: Option<String>,
    muscle: Option<Muscle>,
    category: Option<ExerciseCategory>,
}

pub async fn list_exercises(
    State(app_state): State<Arc<AppState>>,
    ApiQuery(query): ApiQuery<ListExercisesQuery>,
) -> impl IntoResponse {
    let use_case = ListExercisesUseCase {
        exercise_repository: app_state.exercise_repository.clone(),
    };

    let params = ListExercisesUseCaseParams {
        limit: query.limit,
        cursor: query.cursor,
        muscle: query.muscle,
        category: query.category,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn get_exercise(
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = GetExerciseUseCase {
        exercise_repository: app_state.exercise_repository.clone(),
    };

    let params = GetExerciseUseCaseParams { id };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn create_exercise(
    State(app_state): State<Arc<AppState>>,
    _admin: AdminUser,
    ApiJson(payload): ApiJson<ExerciseRequest>,
) -> impl IntoResponse {
    let use_case = CreateExerciseUseCase {
        exercise_repository: app_state.exercise_repository.clone(),
    };

    let params = CreateExerciseUseCaseParams {
        exercise: payload.into(),
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result).with_code(StatusCode::CREATED)
}

pub async fn update_exercise(
    State(app_state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(id): Path<String>,
    ApiJson(payload): ApiJson<ExerciseRequest>,
) -> impl IntoResponse {
    let use_case = UpdateExerciseUseCase {
        exercise_repository: app_state.exercise_repository.clone(),
    };

    let params = UpdateExerciseUseCaseParams {
        id,
        exercise: payload.into(),
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn delete_exercise(
    State(app_state): State<Arc<AppState>>,
    _admin: AdminUser,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let use_case = DeleteExerciseUseCase {
        exercise_repository: app_state.exercise_repository.clone(),
    };

    let params = DeleteExerciseUseCaseParams { id };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
