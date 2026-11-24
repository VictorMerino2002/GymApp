use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::{RoutineRepository, SessionRepository},
        use_cases::routines::routine_input::{RoutineInput, build_session_refs, validate_routine},
    },
    domain::{entities::Routine, value_objects::Error},
};

pub struct UpdateRoutineUseCase {
    pub routine_repository: Arc<dyn RoutineRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct UpdateRoutineUseCaseParams {
    pub id: String,
    pub routine: RoutineInput,
}

impl UpdateRoutineUseCase {
    pub async fn execute(&self, params: UpdateRoutineUseCaseParams) -> Result<Routine, Error> {
        validate_routine(&params.routine)?;

        let mut routine = self
            .routine_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Routine not found"))?;

        let input = params.routine;
        let catalog = self
            .session_repository
            .find_by_ids(&input.session_ids)
            .await?;

        routine.title = input.title.trim().to_string();
        routine.level = input.level;
        routine.duration_weeks = input.duration_weeks;
        routine.description = input.description;
        routine.sessions = build_session_refs(input.session_ids, &catalog)?;

        self.routine_repository.save(&routine).await?;
        Ok(routine)
    }
}
