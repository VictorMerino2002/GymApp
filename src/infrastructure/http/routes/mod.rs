pub mod auth_routes;
pub mod exercise_routes;
pub mod fav_routine_routes;
pub mod routine_routes;
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
        .route(
            "/exercises",
            get(exercise_routes::list_exercises).post(exercise_routes::create_exercise),
        )
        .route(
            "/exercises/{id}",
            get(exercise_routes::get_exercise)
                .put(exercise_routes::update_exercise)
                .delete(exercise_routes::delete_exercise),
        )
        .route(
            "/routines/{id}/favorite",
            post(fav_routine_routes::add_fav_routine)
                .delete(fav_routine_routes::remove_fav_routine),
        )
        .route(
            "/me/favorite-routines",
            get(fav_routine_routes::list_fav_routines),
        )
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
        .route(
            "/routines",
            get(routine_routes::list_routines).post(routine_routes::create_routine),
        )
        .route(
            "/routines/{id}",
            get(routine_routes::get_routine)
                .put(routine_routes::update_routine)
                .delete(routine_routes::delete_routine),
        )
        .route_layer(from_extractor_with_state::<AuthUser, _>(app_state.clone()));

    public_routes.merge(protected_routes).with_state(app_state)
}
