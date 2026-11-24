use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::{FriendshipRepository, UserRepository},
        use_cases::friendships::friendship_errors::to_error,
    },
    domain::{
        entities::{Friendship, FriendshipStatus},
        value_objects::Error,
    },
};

pub struct SendFriendRequestUseCase {
    pub friendship_repository: Arc<dyn FriendshipRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct SendFriendRequestUseCaseParams {
    pub user_id: String,
    pub friend_id: String,
}

impl SendFriendRequestUseCase {
    pub async fn execute(
        &self,
        params: SendFriendRequestUseCaseParams,
    ) -> Result<Friendship, Error> {
        let mut friendship = Friendship::request(params.user_id.clone(), params.friend_id.clone())
            .map_err(to_error)?;

        self.user_repository
            .find_by_id(&params.friend_id)
            .await?
            .ok_or_else(|| Error::not_found("User not found"))?;

        if let Some(mut existing) = self
            .friendship_repository
            .find(&params.user_id, &params.friend_id)
            .await?
        {
            if existing.status() == FriendshipStatus::Accepted {
                return Err(Error::conflict("You are already friends"));
            }
            if existing.requested_by() == params.user_id {
                return Err(Error::conflict("Friend request already sent"));
            }
            existing.accept(&params.user_id).map_err(to_error)?;
            friendship = existing;
        }

        self.friendship_repository.save(&friendship).await?;
        Ok(friendship)
    }
}
