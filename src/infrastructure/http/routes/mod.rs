pub mod auth_routes;
pub mod exercise_routes;
pub mod session_routes;

use std::sync::Arc;

use axum::{
    Router,
    middleware::from_extractor_with_state,
    routing::{get, post},
};

use crate::infrastructure::{bootstrap::AppState, http::extractors::AuthUser};

pub fn router(app_state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/auth/login", post(auth_routes::login))
        .route("/auth/register", post(auth_routes::register));

    let protected_routes = Router::new()
        .route("/exercises", get(exercise_routes::list_exercises))
        .route("/exercises/{id}", get(exercise_routes::get_exercise))
        .route(
            "/sessions",
            get(session_routes::list_sessions).post(session_routes::create_session),
        )
        .route(
            "/sessions/{id}",
            get(session_routes::get_session)
                .put(session_routes::update_session)
                .delete(session_routes::delete_session),
        )
        .route_layer(from_extractor_with_state::<AuthUser, _>(app_state.clone()));

    public_routes.merge(protected_routes).with_state(app_state)
}
