use std::{collections::HashMap, sync::Arc};

use serde::Serialize;

use crate::{
    application::ports::repositories::{FriendshipRepository, UserRepository},
    domain::{
        entities::{FriendshipStatus, PersonalRecord},
        value_objects::Error,
    },
};

#[derive(Debug, Serialize)]
pub struct PublicProfile {
    pub id: String,
    pub username: String,
    pub picture_url: Option<String>,
    pub level: u32,
    pub xp: u32,
    pub streak_weeks: u32,
    pub sessions_done: u32,
    pub exercises_done: u32,
    pub is_me: bool,
    pub friendship: Option<FriendshipSummary>,
    pub personal_records: Option<HashMap<String, PersonalRecord>>,
}

#[derive(Debug, Serialize)]
pub struct FriendshipSummary {
    pub status: FriendshipStatus,
    pub requested_by_me: bool,
}

pub struct GetUserProfileUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub friendship_repository: Arc<dyn FriendshipRepository>,
}

pub struct GetUserProfileUseCaseParams {
    pub viewer_id: String,
    pub user_id: String,
}

impl GetUserProfileUseCase {
    pub async fn execute(
        &self,
        params: GetUserProfileUseCaseParams,
    ) -> Result<PublicProfile, Error> {
        let user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::not_found("User not found"))?;

        let is_me = params.viewer_id == user.id;
        let friendship = if is_me {
            None
        } else {
            self.friendship_repository
                .find(&params.viewer_id, &user.id)
                .await?
                .map(|friendship| FriendshipSummary {
                    status: friendship.status(),
                    requested_by_me: friendship.requested_by() == params.viewer_id,
                })
        };

        let can_see_records = is_me
            || friendship
                .as_ref()
                .is_some_and(|friendship| friendship.status == FriendshipStatus::Accepted);

        Ok(PublicProfile {
            level: user.stats.level(),
            xp: user.stats.xp,
            streak_weeks: user.stats.streak_weeks,
            sessions_done: user.stats.sessions_done,
            exercises_done: user.stats.exercises_done,
            id: user.id,
            username: user.username,
            picture_url: user.picture_url,
            is_me,
            friendship,
            personal_records: can_see_records.then_some(user.personal_records),
        })
    }
}
