use std::sync::Arc;

use crate::{
    application::ports::repositories::RoutineRepository,
    domain::{entities::Routine, value_objects::Error},
};

pub struct GetRoutineUseCase {
    pub routine_repository: Arc<dyn RoutineRepository>,
}

pub struct GetRoutineUseCaseParams {
    pub id: String,
}

impl GetRoutineUseCase {
    pub async fn execute(&self, params: GetRoutineUseCaseParams) -> Result<Routine, Error> {
        self.routine_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Routine not found"))
    }
}
