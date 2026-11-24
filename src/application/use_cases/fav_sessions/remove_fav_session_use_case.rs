use std::sync::Arc;

use crate::{application::ports::repositories::FavSessionRepository, domain::value_objects::Error};

pub struct RemoveFavSessionUseCase {
    pub fav_session_repository: Arc<dyn FavSessionRepository>,
}

pub struct RemoveFavSessionUseCaseParams {
    pub user_id: String,
    pub session_id: String,
}

impl RemoveFavSessionUseCase {
    pub async fn execute(&self, params: RemoveFavSessionUseCaseParams) -> Result<(), Error> {
        self.fav_session_repository
            .delete(&params.user_id, &params.session_id)
            .await
    }
}
