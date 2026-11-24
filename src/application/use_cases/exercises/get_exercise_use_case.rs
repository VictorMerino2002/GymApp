use std::sync::Arc;

use crate::{
    application::ports::repositories::ExerciseRepository,
    domain::{entities::Exercise, value_objects::Error},
};

pub struct GetExerciseUseCase {
    pub exercise_repository: Arc<dyn ExerciseRepository>,
}

pub struct GetExerciseUseCaseParams {
    pub id: String,
}

impl GetExerciseUseCase {
    pub async fn execute(&self, params: GetExerciseUseCaseParams) -> Result<Exercise, Error> {
        self.exercise_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Exercise not found"))
    }
}
