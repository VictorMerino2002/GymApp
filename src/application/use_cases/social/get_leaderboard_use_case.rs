use std::sync::Arc;

use serde::Serialize;

use crate::{
    application::{
        ports::repositories::{FriendshipRepository, UserRepository},
        use_cases::social::friend_ids::accepted_friend_ids,
    },
    domain::value_objects::Error,
};

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub user_id: String,
    pub username: String,
    pub picture_url: Option<String>,
    pub level: u32,
    pub xp: u32,
    pub streak_weeks: u32,
    pub is_me: bool,
}

pub struct GetLeaderboardUseCase {
    pub friendship_repository: Arc<dyn FriendshipRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct GetLeaderboardUseCaseParams {
    pub user_id: String,
}

impl GetLeaderboardUseCase {
    pub async fn execute(
        &self,
        params: GetLeaderboardUseCaseParams,
    ) -> Result<Vec<LeaderboardEntry>, Error> {
        let mut ids = accepted_friend_ids(&*self.friendship_repository, &params.user_id).await?;
        ids.push(params.user_id.clone());

        let mut users = self.user_repository.find_by_ids(&ids).await?;
        users.sort_by(|a, b| {
            b.stats
                .xp
                .cmp(&a.stats.xp)
                .then_with(|| a.username.to_lowercase().cmp(&b.username.to_lowercase()))
        });

        let mut entries: Vec<LeaderboardEntry> = Vec::with_capacity(users.len());
        for (position, user) in users.into_iter().enumerate() {
            let rank = match entries.last() {
                Some(previous) if previous.xp == user.stats.xp => previous.rank,
                _ => position as u32 + 1,
            };

            entries.push(LeaderboardEntry {
                rank,
                is_me: user.id == params.user_id,
                level: user.stats.level(),
                xp: user.stats.xp,
                streak_weeks: user.stats.streak_weeks,
                user_id: user.id,
                username: user.username,
                picture_url: user.picture_url,
            });
        }

        Ok(entries)
    }
}
