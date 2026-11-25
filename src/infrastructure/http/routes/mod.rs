pub mod auth_routes;
pub mod exercise_history_routes;
pub mod exercise_routes;
pub mod fav_routine_routes;
pub mod fav_session_routes;
pub mod friendship_routes;
pub mod me_routes;
pub mod routine_routes;
pub mod session_routes;
pub mod social_routes;
pub mod user_routes;

use std::sync::Arc;

use axum::{
    Router,
    middleware::from_extractor_with_state,
    routing::{get, post, put},
};

use crate::infrastructure::{bootstrap::AppState, http::extractors::AuthUser};

pub fn router(app_state: Arc<AppState>) -> Router {
    let public_routes = Router::new()
        .route("/auth/login", post(auth_routes::login))
        .route("/auth/register", post(auth_routes::register))
        .route("/auth/refresh", post(auth_routes::refresh));

    let protected_routes = Router::new()
        .route("/auth/logout", post(auth_routes::logout))
        .route(
            "/me",
            get(me_routes::get_profile)
                .patch(me_routes::update_profile)
                .delete(me_routes::delete_account),
        )
        .route("/me/password", post(me_routes::change_password))
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
        .route(
            "/sessions/{id}/complete",
            post(session_routes::complete_session),
        )
        .route(
            "/sessions/{id}/favorite",
            post(fav_session_routes::add_fav_session)
                .delete(fav_session_routes::remove_fav_session),
        )
        .route(
            "/me/favorite-sessions",
            get(fav_session_routes::list_fav_sessions),
        )
        .route(
            "/friends/{user_id}",
            post(friendship_routes::send_friend_request)
                .delete(friendship_routes::remove_friendship),
        )
        .route(
            "/friends/{user_id}/accept",
            post(friendship_routes::accept_friend_request),
        )
        .route("/me/friends", get(friendship_routes::list_friends))
        .route("/users", get(user_routes::list_users))
        .route("/users/{id}", get(social_routes::get_user_profile))
        .route("/users/{id}/role", put(user_routes::change_user_role))
        .route("/me/feed", get(social_routes::get_feed))
        .route("/me/leaderboard", get(social_routes::get_leaderboard))
        .route(
            "/me/exercise-history",
            get(exercise_history_routes::list_exercise_history)
                .post(exercise_history_routes::create_exercise_history),
        )
        .route(
            "/me/exercise-history/{id}",
            get(exercise_history_routes::get_exercise_history)
                .delete(exercise_history_routes::delete_exercise_history),
        )
        .route_layer(from_extractor_with_state::<AuthUser, _>(app_state.clone()));

    public_routes.merge(protected_routes).with_state(app_state)
}
