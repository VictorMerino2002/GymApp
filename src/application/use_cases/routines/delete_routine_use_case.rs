use std::sync::Arc;

use crate::{application::ports::repositories::RoutineRepository, domain::value_objects::Error};

pub struct DeleteRoutineUseCase {
    pub routine_repository: Arc<dyn RoutineRepository>,
}

pub struct DeleteRoutineUseCaseParams {
    pub id: String,
}

impl DeleteRoutineUseCase {
    pub async fn execute(&self, params: DeleteRoutineUseCaseParams) -> Result<(), Error> {
        self.routine_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Routine not found"))?;

        self.routine_repository.delete(&params.id).await
    }
}
