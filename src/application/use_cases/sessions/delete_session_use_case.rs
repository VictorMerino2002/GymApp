use std::sync::Arc;

use crate::{application::ports::repositories::SessionRepository, domain::value_objects::Error};

pub struct DeleteSessionUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct DeleteSessionUseCaseParams {
    pub user_id: String,
    pub id: String,
}

impl DeleteSessionUseCase {
    pub async fn execute(&self, params: DeleteSessionUseCaseParams) -> Result<(), Error> {
        let session = self
            .session_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Session not found"))?;

        if !session.is_authored_by(&params.user_id) {
            return Err(Error::forbidden("You can only delete your own sessions"));
        }

        self.session_repository.delete(&session.id).await
    }
}
