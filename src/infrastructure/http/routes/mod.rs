pub mod auth_routes;

use std::sync::Arc;

use axum::{Router, routing::post};

use crate::infrastructure::bootstrap::AppState;

pub fn router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/auth/login", post(auth_routes::login))
        .route("/auth/register", post(auth_routes::register))
        .with_state(app_state)
}
