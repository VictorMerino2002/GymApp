use std::sync::Arc;

use crate::{
    application::ports::repositories::ExerciseHistoryRepository,
    domain::{
        entities::ExerciseHistory,
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListExerciseHistoryUseCase {
    pub exercise_history_repository: Arc<dyn ExerciseHistoryRepository>,
}

pub struct ListExerciseHistoryUseCaseParams {
    pub user_id: String,
    pub exercise_id: Option<String>,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListExerciseHistoryUseCase {
    pub async fn execute(
        &self,
        params: ListExerciseHistoryUseCaseParams,
    ) -> Result<Page<ExerciseHistory>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        self.exercise_history_repository
            .find_by_user(
                &params.user_id,
                params.exercise_id.as_deref(),
                limit,
                params.cursor.as_deref(),
            )
            .await
    }
}
