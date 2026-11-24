use std::sync::Arc;

use crate::{
    application::ports::repositories::{FavRoutineRepository, RoutineRepository},
    domain::{entities::FavRoutine, value_objects::Error},
};

pub struct AddFavRoutineUseCase {
    pub fav_routine_repository: Arc<dyn FavRoutineRepository>,
    pub routine_repository: Arc<dyn RoutineRepository>,
}

pub struct AddFavRoutineUseCaseParams {
    pub user_id: String,
    pub routine_id: String,
}

impl AddFavRoutineUseCase {
    pub async fn execute(&self, params: AddFavRoutineUseCaseParams) -> Result<(), Error> {
        self.routine_repository
            .find_by_id(&params.routine_id)
            .await?
            .ok_or_else(|| Error::not_found("Routine not found"))?;

        let fav_routine = FavRoutine {
            user_id: params.user_id,
            routine_id: params.routine_id,
        };

        self.fav_routine_repository.save(&fav_routine).await
    }
}
