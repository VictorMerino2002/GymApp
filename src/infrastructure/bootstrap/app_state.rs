use std::sync::Arc;

use crate::{
    application::ports::{
        JwtEncoderPort,
        repositories::{
            ExerciseRepository, FavRoutineRepository, RoutineRepository, SessionRepository,
            UserRepository,
        },
    },
    infrastructure::adapters::{
        JwtEncoder,
        repositories::{
            DynamodbExerciseRepository, DynamodbFavRoutineRepository, DynamodbRoutineRepository,
            DynamodbSessionRepository, DynamodbUserRepository,
        },
    },
};

pub struct AppState {
    pub user_repository: Arc<dyn UserRepository>,
    pub exercise_repository: Arc<dyn ExerciseRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub routine_repository: Arc<dyn RoutineRepository>,
    pub fav_routine_repository: Arc<dyn FavRoutineRepository>,
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
}

impl AppState {
    pub async fn default() -> Self {
        let aws_config = aws_config::load_from_env().await;
        let dynamodb = aws_sdk_dynamodb::Client::new(&aws_config);
        let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE env var not set");
        let exercises_table =
            std::env::var("EXERCISES_TABLE").expect("EXERCISES_TABLE env var not set");
        let sessions_table =
            std::env::var("SESSIONS_TABLE").expect("SESSIONS_TABLE env var not set");
        let routines_table =
            std::env::var("ROUTINES_TABLE").expect("ROUTINES_TABLE env var not set");
        let fav_routines_table =
            std::env::var("FAV_ROUTINES_TABLE").expect("FAV_ROUTINES_TABLE env var not set");

        let user_repository = DynamodbUserRepository::new(dynamodb.clone(), users_table);
        let exercise_repository =
            DynamodbExerciseRepository::new(dynamodb.clone(), exercises_table);
        let session_repository = DynamodbSessionRepository::new(dynamodb.clone(), sessions_table);
        let routine_repository = DynamodbRoutineRepository::new(dynamodb.clone(), routines_table);
        let fav_routine_repository =
            DynamodbFavRoutineRepository::new(dynamodb, fav_routines_table);
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET env var not set");
        let jwt_encoder = JwtEncoder::new(jwt_secret);

        Self {
            user_repository: Arc::new(user_repository),
            exercise_repository: Arc::new(exercise_repository),
            session_repository: Arc::new(session_repository),
            routine_repository: Arc::new(routine_repository),
            fav_routine_repository: Arc::new(fav_routine_repository),
            jwt_encoder: Arc::new(jwt_encoder),
        }
    }
}
