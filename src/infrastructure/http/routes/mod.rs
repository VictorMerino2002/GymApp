pub mod auth_routes;
pub mod exercise_routes;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::infrastructure::bootstrap::AppState;

pub fn router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/auth/login", post(auth_routes::login))
        .route("/auth/register", post(auth_routes::register))
        .route("/exercises", get(exercise_routes::list_exercises))
        .route("/exercises/{id}", get(exercise_routes::get_exercise))
        .with_state(app_state)
}
