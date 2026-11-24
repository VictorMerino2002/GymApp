use std::sync::Arc;

use crate::{application::ports::repositories::FavRoutineRepository, domain::value_objects::Error};

pub struct RemoveFavRoutineUseCase {
    pub fav_routine_repository: Arc<dyn FavRoutineRepository>,
}

pub struct RemoveFavRoutineUseCaseParams {
    pub user_id: String,
    pub routine_id: String,
}

impl RemoveFavRoutineUseCase {
    pub async fn execute(&self, params: RemoveFavRoutineUseCaseParams) -> Result<(), Error> {
        self.fav_routine_repository
            .delete(&params.user_id, &params.routine_id)
            .await
    }
}
