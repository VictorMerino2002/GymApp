use std::sync::Arc;

use crate::{application::ports::repositories::SessionRepository, domain::value_objects::Error};

pub struct DeleteSessionUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct DeleteSessionUseCaseParams {
    pub id: String,
}

impl DeleteSessionUseCase {
    pub async fn execute(&self, params: DeleteSessionUseCaseParams) -> Result<(), Error> {
        self.session_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))?;

        self.session_repository.delete(&params.id).await
    }
}
