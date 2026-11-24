use std::sync::Arc;

use crate::{
    application::ports::{
        JwtEncoderPort, PasswordHasherPort,
        repositories::{
            ActivityRepository, ExerciseHistoryRepository, ExerciseRepository,
            FavRoutineRepository, FavSessionRepository, FriendshipRepository, RoutineRepository,
            SessionRepository, UserRepository,
        },
    },
    infrastructure::adapters::{
        Argon2PasswordHasher, JwtEncoder,
        repositories::{
            DynamodbActivityRepository, DynamodbExerciseHistoryRepository,
            DynamodbExerciseRepository, DynamodbFavRoutineRepository, DynamodbFavSessionRepository,
            DynamodbFriendshipRepository, DynamodbRoutineRepository, DynamodbSessionRepository,
            DynamodbUserRepository,
        },
    },
};

pub struct AppState {
    pub user_repository: Arc<dyn UserRepository>,
    pub activity_repository: Arc<dyn ActivityRepository>,
    pub exercise_repository: Arc<dyn ExerciseRepository>,
    pub exercise_history_repository: Arc<dyn ExerciseHistoryRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub routine_repository: Arc<dyn RoutineRepository>,
    pub fav_routine_repository: Arc<dyn FavRoutineRepository>,
    pub fav_session_repository: Arc<dyn FavSessionRepository>,
    pub friendship_repository: Arc<dyn FriendshipRepository>,
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
}

impl AppState {
    pub async fn default() -> Self {
        let aws_config = aws_config::load_from_env().await;
        let dynamodb = aws_sdk_dynamodb::Client::new(&aws_config);
        let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE env var not set");
        let activities_table =
            std::env::var("ACTIVITIES_TABLE").expect("ACTIVITIES_TABLE env var not set");
        let exercises_table =
            std::env::var("EXERCISES_TABLE").expect("EXERCISES_TABLE env var not set");
        let exercise_history_table = std::env::var("EXERCISE_HISTORY_TABLE")
            .expect("EXERCISE_HISTORY_TABLE env var not set");
        let sessions_table =
            std::env::var("SESSIONS_TABLE").expect("SESSIONS_TABLE env var not set");
        let routines_table =
            std::env::var("ROUTINES_TABLE").expect("ROUTINES_TABLE env var not set");
        let fav_routines_table =
            std::env::var("FAV_ROUTINES_TABLE").expect("FAV_ROUTINES_TABLE env var not set");
        let fav_sessions_table =
            std::env::var("FAV_SESSIONS_TABLE").expect("FAV_SESSIONS_TABLE env var not set");
        let friendships_table =
            std::env::var("FRIENDSHIPS_TABLE").expect("FRIENDSHIPS_TABLE env var not set");

        let user_repository = DynamodbUserRepository::new(dynamodb.clone(), users_table);
        let activity_repository =
            DynamodbActivityRepository::new(dynamodb.clone(), activities_table);
        let exercise_repository =
            DynamodbExerciseRepository::new(dynamodb.clone(), exercises_table);
        let exercise_history_repository =
            DynamodbExerciseHistoryRepository::new(dynamodb.clone(), exercise_history_table);
        let session_repository = DynamodbSessionRepository::new(dynamodb.clone(), sessions_table);
        let routine_repository = DynamodbRoutineRepository::new(dynamodb.clone(), routines_table);
        let fav_routine_repository =
            DynamodbFavRoutineRepository::new(dynamodb.clone(), fav_routines_table);
        let fav_session_repository =
            DynamodbFavSessionRepository::new(dynamodb.clone(), fav_sessions_table);
        let friendship_repository = DynamodbFriendshipRepository::new(dynamodb, friendships_table);
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET env var not set");
        let jwt_encoder = JwtEncoder::new(jwt_secret);

        Self {
            user_repository: Arc::new(user_repository),
            activity_repository: Arc::new(activity_repository),
            exercise_repository: Arc::new(exercise_repository),
            exercise_history_repository: Arc::new(exercise_history_repository),
            session_repository: Arc::new(session_repository),
            routine_repository: Arc::new(routine_repository),
            fav_routine_repository: Arc::new(fav_routine_repository),
            fav_session_repository: Arc::new(fav_session_repository),
            friendship_repository: Arc::new(friendship_repository),
            jwt_encoder: Arc::new(jwt_encoder),
            password_hasher: Arc::new(Argon2PasswordHasher),
        }
    }
}
