use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::exercises::{
        GetExerciseUseCase, GetExerciseUseCaseParams, ListExercisesUseCase,
        ListExercisesUseCaseParams,
    },
    infrastructure::{
        bootstrap::AppState,
        http::{extractors::ApiQuery, value_objects::ApiResponse},
    },
};

#[derive(Deserialize)]
pub struct ListExercisesQuery {
    limit: Option<u16>,
    cursor: Option<String>,
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
