use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::{ExerciseRepository, SessionRepository, UserRepository},
        use_cases::sessions::session_exercises::{
            SessionExerciseInput, build_session_exercises, exercise_ids, validate_session,
        },
    },
    domain::{
        entities::Session,
        value_objects::{Author, Error},
    },
};

pub struct CreateSessionUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
    pub exercise_repository: Arc<dyn ExerciseRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct CreateSessionUseCaseParams {
    pub user_id: String,
    pub name: String,
    pub exercises: Vec<SessionExerciseInput>,
}

impl CreateSessionUseCase {
    pub async fn execute(&self, params: CreateSessionUseCaseParams) -> Result<Session, Error> {
        validate_session(&params.name, &params.exercises)?;

        let user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::unauthorized("User not found"))?;

        let catalog = self
            .exercise_repository
            .find_by_ids(&exercise_ids(&params.exercises))
            .await?;
        let exercises = build_session_exercises(params.exercises, &catalog)?;

        let author = Author {
            user_id: user.id,
            user_name: user.username,
        };
        let session = Session::new(params.name.trim(), author, exercises);

        self.session_repository.save(&session).await?;
        Ok(session)
    }
}
