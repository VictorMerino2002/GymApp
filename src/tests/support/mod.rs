mod activity_repository;
mod exercise_history_repository;
mod exercise_repository;
mod fav_routine_repository;
mod fav_session_repository;
mod friendship_repository;
mod password_hasher;
mod routine_repository;
mod session_repository;
mod user_repository;

use std::sync::Arc;

use crate::{
    domain::{
        entities::{Exercise, Session, User},
        value_objects::{Author, ExerciseCategory, Muscle},
    },
    infrastructure::{adapters::JwtEncoder, bootstrap::AppState},
};

pub use activity_repository::InMemoryActivityRepository;
pub use exercise_history_repository::InMemoryExerciseHistoryRepository;
pub use exercise_repository::InMemoryExerciseRepository;
pub use fav_routine_repository::InMemoryFavRoutineRepository;
pub use fav_session_repository::InMemoryFavSessionRepository;
pub use friendship_repository::InMemoryFriendshipRepository;
pub use password_hasher::FakePasswordHasher;
pub use routine_repository::InMemoryRoutineRepository;
pub use session_repository::InMemorySessionRepository;
pub use user_repository::InMemoryUserRepository;

pub const PASSWORD: &str = "password123";
pub const JWT_SECRET: &str = "test-secret";

pub fn user(id: &str, username: &str) -> User {
    let mut user = User::new(username, format!("hash:{PASSWORD}"));
    user.id = id.to_string();
    user
}

pub fn exercise(id: &str, name: &str, muscles: Vec<Muscle>) -> Exercise {
    let mut exercise = Exercise::new(
        name,
        ExerciseCategory::Strength,
        muscles,
        "",
        vec![],
        None,
        None,
    );
    exercise.id = id.to_string();
    exercise
}

pub fn session(id: &str, name: &str) -> Session {
    let author = Author {
        user_id: "admin".to_string(),
        user_name: "admin".to_string(),
    };
    let mut session = Session::new(name, author, vec![], vec![]);
    session.id = id.to_string();
    session
}

pub fn jwt_encoder() -> Arc<JwtEncoder> {
    Arc::new(JwtEncoder::new(JWT_SECRET.to_string()))
}

pub fn app_state(user_repository: Arc<InMemoryUserRepository>) -> Arc<AppState> {
    Arc::new(AppState {
        user_repository,
        activity_repository: Arc::new(InMemoryActivityRepository::default()),
        exercise_repository: Arc::new(InMemoryExerciseRepository::default()),
        exercise_history_repository: Arc::new(InMemoryExerciseHistoryRepository::default()),
        session_repository: Arc::new(InMemorySessionRepository::default()),
        routine_repository: Arc::new(InMemoryRoutineRepository::default()),
        fav_routine_repository: Arc::new(InMemoryFavRoutineRepository::default()),
        fav_session_repository: Arc::new(InMemoryFavSessionRepository::default()),
        friendship_repository: Arc::new(InMemoryFriendshipRepository::default()),
        jwt_encoder: jwt_encoder(),
        password_hasher: Arc::new(FakePasswordHasher::default()),
    })
}
