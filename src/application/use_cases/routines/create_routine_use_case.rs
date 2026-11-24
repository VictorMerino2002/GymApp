use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::{RoutineRepository, SessionRepository, UserRepository},
        use_cases::routines::routine_input::{RoutineInput, build_session_refs, validate_routine},
    },
    domain::{
        entities::Routine,
        value_objects::{Author, Error},
    },
};

pub struct CreateRoutineUseCase {
    pub routine_repository: Arc<dyn RoutineRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct CreateRoutineUseCaseParams {
    pub user_id: String,
    pub routine: RoutineInput,
}

impl CreateRoutineUseCase {
    pub async fn execute(&self, params: CreateRoutineUseCaseParams) -> Result<Routine, Error> {
        validate_routine(&params.routine)?;

        let user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::unauthorized("User not found"))?;

        let input = params.routine;
        let catalog = self
            .session_repository
            .find_by_ids(&input.session_ids)
            .await?;
        let sessions = build_session_refs(input.session_ids, &catalog)?;

        let author = Author {
            user_id: user.id,
            user_name: user.username,
        };
        let routine = Routine::new(
            input.title.trim(),
            author,
            input.level,
            input.duration_weeks,
            input.description,
            sessions,
        );

        self.routine_repository.save(&routine).await?;
        Ok(routine)
    }
}
