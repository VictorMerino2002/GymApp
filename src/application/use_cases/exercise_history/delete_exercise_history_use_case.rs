use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::{ExerciseHistoryRepository, UserRepository},
        use_cases::user_progress::update_user_progress,
    },
    domain::value_objects::Error,
};

pub struct DeleteExerciseHistoryUseCase {
    pub exercise_history_repository: Arc<dyn ExerciseHistoryRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct DeleteExerciseHistoryUseCaseParams {
    pub user_id: String,
    pub id: String,
}

impl DeleteExerciseHistoryUseCase {
    pub async fn execute(&self, params: DeleteExerciseHistoryUseCaseParams) -> Result<(), Error> {
        let history = self
            .exercise_history_repository
            .find_by_id(&params.user_id, &params.id)
            .await?
            .ok_or_else(|| Error::not_found("Exercise history not found"))?;

        self.exercise_history_repository
            .delete(&params.user_id, &params.id)
            .await?;

        update_user_progress(&*self.user_repository, &params.user_id, |user| {
            user.undo_exercise(history.xp_awarded)
        })
        .await
    }
}
