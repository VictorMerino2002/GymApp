use std::sync::Arc;

use crate::{
    application::ports::repositories::ExerciseHistoryRepository,
    domain::{entities::ExerciseHistory, value_objects::Error},
};

pub struct GetExerciseHistoryUseCase {
    pub exercise_history_repository: Arc<dyn ExerciseHistoryRepository>,
}

pub struct GetExerciseHistoryUseCaseParams {
    pub user_id: String,
    pub id: String,
}

impl GetExerciseHistoryUseCase {
    pub async fn execute(
        &self,
        params: GetExerciseHistoryUseCaseParams,
    ) -> Result<ExerciseHistory, Error> {
        self.exercise_history_repository
            .find_by_id(&params.user_id, &params.id)
            .await?
            .ok_or_else(|| Error::not_found("Exercise history not found"))
    }
}
