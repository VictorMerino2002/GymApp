use std::sync::Arc;

use crate::{application::ports::repositories::ExerciseRepository, domain::value_objects::Error};

pub struct DeleteExerciseUseCase {
    pub exercise_repository: Arc<dyn ExerciseRepository>,
}

pub struct DeleteExerciseUseCaseParams {
    pub id: String,
}

impl DeleteExerciseUseCase {
    pub async fn execute(&self, params: DeleteExerciseUseCaseParams) -> Result<(), Error> {
        self.exercise_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Exercise not found"))?;

        self.exercise_repository.delete(&params.id).await
    }
}
