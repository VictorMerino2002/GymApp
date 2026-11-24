use std::sync::Arc;

use crate::{
    application::ports::repositories::{FavSessionRepository, SessionRepository},
    domain::{entities::FavSession, value_objects::Error},
};

pub struct AddFavSessionUseCase {
    pub fav_session_repository: Arc<dyn FavSessionRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct AddFavSessionUseCaseParams {
    pub user_id: String,
    pub session_id: String,
}

impl AddFavSessionUseCase {
    pub async fn execute(&self, params: AddFavSessionUseCaseParams) -> Result<(), Error> {
        self.session_repository
            .find_by_id(&params.session_id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))?;

        let fav_session = FavSession {
            user_id: params.user_id,
            session_id: params.session_id,
        };

        self.fav_session_repository.save(&fav_session).await
    }
}
