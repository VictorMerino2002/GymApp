use std::sync::Arc;

use crate::{
    application::{ports::repositories::UserRepository, use_cases::me::Profile},
    domain::value_objects::Error,
};

pub struct GetProfileUseCase {
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct GetProfileUseCaseParams {
    pub user_id: String,
}

impl GetProfileUseCase {
    pub async fn execute(&self, params: GetProfileUseCaseParams) -> Result<Profile, Error> {
        self.user_repository
            .find_by_id(&params.user_id)
            .await?
            .map(Profile::from)
            .ok_or_else(|| Error::unauthorized("User not found"))
    }
}
