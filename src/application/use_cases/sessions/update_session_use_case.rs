use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::{ExerciseRepository, SessionRepository},
        use_cases::sessions::session_exercises::{
            SessionExerciseInput, build_session_exercises, exercise_ids, validate_session,
        },
    },
    domain::{entities::Session, value_objects::Error},
};

pub struct UpdateSessionUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
    pub exercise_repository: Arc<dyn ExerciseRepository>,
}

pub struct UpdateSessionUseCaseParams {
    pub id: String,
    pub name: String,
    pub exercises: Vec<SessionExerciseInput>,
}

impl UpdateSessionUseCase {
    pub async fn execute(&self, params: UpdateSessionUseCaseParams) -> Result<Session, Error> {
        validate_session(&params.name, &params.exercises)?;

        let mut session = self
            .session_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))?;

        let catalog = self
            .exercise_repository
            .find_by_ids(&exercise_ids(&params.exercises))
            .await?;

        session.name = params.name.trim().to_string();
        session.exercises = build_session_exercises(params.exercises, &catalog)?;

        self.session_repository.save(&session).await?;
        Ok(session)
    }
}
