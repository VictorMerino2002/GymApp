use std::sync::Arc;

use crate::{
    application::ports::repositories::SessionRepository,
    domain::{entities::Session, value_objects::Error},
};

pub struct GetSessionUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct GetSessionUseCaseParams {
    pub id: String,
}

impl GetSessionUseCase {
    pub async fn execute(&self, params: GetSessionUseCaseParams) -> Result<Session, Error> {
        self.session_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))
    }
}
