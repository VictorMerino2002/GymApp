use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::{
    application::ports::repositories::{FriendshipRepository, UserRepository},
    domain::{
        entities::{FriendshipStatus, User},
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

#[derive(Debug, Serialize)]
pub struct Friend {
    pub user_id: String,
    pub username: String,
    pub picture_url: Option<String>,
    pub status: FriendshipStatus,
    pub requested_by_me: bool,
}

pub struct ListFriendsUseCase {
    pub friendship_repository: Arc<dyn FriendshipRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct ListFriendsUseCaseParams {
    pub user_id: String,
    pub status: Option<FriendshipStatus>,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListFriendsUseCase {
    pub async fn execute(&self, params: ListFriendsUseCaseParams) -> Result<Page<Friend>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let friendships = self
            .friendship_repository
            .find_by_user(
                &params.user_id,
                params.status,
                limit,
                params.cursor.as_deref(),
            )
            .await?;

        let friend_id = |user1_id: &str, user2_id: &str| -> String {
            if user1_id == params.user_id {
                user2_id.to_string()
            } else {
                user1_id.to_string()
            }
        };

        let friend_ids: Vec<String> = friendships
            .items
            .iter()
            .map(|friendship| friend_id(friendship.user1_id(), friendship.user2_id()))
            .collect();

        let users: HashMap<String, User> = self
            .user_repository
            .find_by_ids(&friend_ids)
            .await?
            .into_iter()
            .map(|user| (user.id.clone(), user))
            .collect();

        let items = friendships
            .items
            .iter()
            .zip(friend_ids)
            .filter_map(|(friendship, friend_id)| {
                let user = users.get(&friend_id)?;
                Some(Friend {
                    user_id: friend_id,
                    username: user.username.clone(),
                    picture_url: user.picture_url.clone(),
                    status: friendship.status(),
                    requested_by_me: friendship.requested_by() == params.user_id,
                })
            })
            .collect();

        Ok(Page {
            items,
            next_cursor: friendships.next_cursor,
        })
    }
}
