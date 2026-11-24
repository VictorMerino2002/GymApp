use std::sync::Arc;

use crate::{
    application::{ports::repositories::UserRepository, use_cases::me::Profile},
    domain::{
        entities::{validate_picture_url, validate_username},
        value_objects::Error,
    },
};

pub struct UpdateProfileUseCase {
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct UpdateProfileUseCaseParams {
    pub user_id: String,
    pub username: Option<String>,
    pub picture_url: Option<Option<String>>,
}

impl UpdateProfileUseCase {
    pub async fn execute(&self, params: UpdateProfileUseCaseParams) -> Result<Profile, Error> {
        if let Some(username) = &params.username {
            validate_username(username)?;
        }

        if let Some(Some(url)) = &params.picture_url {
            validate_picture_url(url)?;
        }

        let mut user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::unauthorized("User not found"))?;

        let previous_username = user.username.clone();

        if let Some(username) = params.username {
            user.username = username;
        }

        if let Some(picture_url) = params.picture_url {
            user.picture_url = picture_url;
        }

        self.user_repository
            .update_profile(&user, &previous_username)
            .await?;

        Ok(Profile::from(user))
    }
}
