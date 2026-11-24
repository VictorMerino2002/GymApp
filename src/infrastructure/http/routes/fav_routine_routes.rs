use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    application::use_cases::fav_routines::{
        AddFavRoutineUseCase, AddFavRoutineUseCaseParams, ListFavRoutinesUseCase,
        ListFavRoutinesUseCaseParams, RemoveFavRoutineUseCase, RemoveFavRoutineUseCaseParams,
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
pub struct ListFavRoutinesQuery {
    limit: Option<u16>,
    cursor: Option<String>,
}

pub async fn add_fav_routine(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(routine_id): Path<String>,
) -> impl IntoResponse {
    let use_case = AddFavRoutineUseCase {
        fav_routine_repository: app_state.fav_routine_repository.clone(),
        routine_repository: app_state.routine_repository.clone(),
    };

    let params = AddFavRoutineUseCaseParams {
        user_id: user.user_id,
        routine_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn remove_fav_routine(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    Path(routine_id): Path<String>,
) -> impl IntoResponse {
    let use_case = RemoveFavRoutineUseCase {
        fav_routine_repository: app_state.fav_routine_repository.clone(),
    };

    let params = RemoveFavRoutineUseCaseParams {
        user_id: user.user_id,
        routine_id,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}

pub async fn list_fav_routines(
    State(app_state): State<Arc<AppState>>,
    user: AuthUser,
    ApiQuery(query): ApiQuery<ListFavRoutinesQuery>,
) -> impl IntoResponse {
    let use_case = ListFavRoutinesUseCase {
        fav_routine_repository: app_state.fav_routine_repository.clone(),
        routine_repository: app_state.routine_repository.clone(),
    };

    let params = ListFavRoutinesUseCaseParams {
        user_id: user.user_id,
        limit: query.limit,
        cursor: query.cursor,
    };

    let result = use_case.execute(params).await;
    ApiResponse::default(result)
}
