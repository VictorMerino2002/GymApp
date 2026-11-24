use std::sync::Arc;

use crate::{application::ports::repositories::FriendshipRepository, domain::value_objects::Error};

pub struct RemoveFriendshipUseCase {
    pub friendship_repository: Arc<dyn FriendshipRepository>,
}

pub struct RemoveFriendshipUseCaseParams {
    pub user_id: String,
    pub friend_id: String,
}

impl RemoveFriendshipUseCase {
    pub async fn execute(&self, params: RemoveFriendshipUseCaseParams) -> Result<(), Error> {
        self.friendship_repository
            .find(&params.user_id, &params.friend_id)
            .await?
            .ok_or_else(|| Error::not_found("Friendship not found"))?;

        self.friendship_repository
            .delete(&params.user_id, &params.friend_id)
            .await
    }
}
