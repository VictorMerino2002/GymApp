use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::FriendshipRepository,
        use_cases::friendships::friendship_errors::to_error,
    },
    domain::{entities::Friendship, value_objects::Error},
};

pub struct AcceptFriendRequestUseCase {
    pub friendship_repository: Arc<dyn FriendshipRepository>,
}

pub struct AcceptFriendRequestUseCaseParams {
    pub user_id: String,
    pub friend_id: String,
}

impl AcceptFriendRequestUseCase {
    pub async fn execute(
        &self,
        params: AcceptFriendRequestUseCaseParams,
    ) -> Result<Friendship, Error> {
        let mut friendship = self
            .friendship_repository
            .find(&params.user_id, &params.friend_id)
            .await?
            .ok_or_else(|| Error::not_found("Friend request not found"))?;

        friendship.accept(&params.user_id).map_err(to_error)?;

        self.friendship_repository.save(&friendship).await?;
        Ok(friendship)
    }
}
