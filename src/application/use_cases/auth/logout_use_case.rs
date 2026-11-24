use std::sync::Arc;

use crate::{application::ports::repositories::UserRepository, domain::value_objects::Error};

pub struct LogoutUseCase {
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct LogoutUseCaseParams {
    pub user_id: String,
}

impl LogoutUseCase {
    pub async fn execute(&self, params: LogoutUseCaseParams) -> Result<(), Error> {
        self.user_repository
            .increment_token_version(&params.user_id)
            .await
    }
}
