use std::{collections::HashMap, sync::Arc};

use serde::Serialize;
use tokio::task::JoinSet;

use crate::{
    application::{
        ports::repositories::{ActivityRepository, FriendshipRepository, UserRepository},
        use_cases::social::friend_ids::accepted_friend_ids,
    },
    domain::{
        entities::{Activity, User},
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 50;

#[derive(Debug, Serialize)]
pub struct FeedItem {
    #[serde(flatten)]
    pub activity: Activity,
    pub username: String,
    pub picture_url: Option<String>,
}

pub struct GetFeedUseCase {
    pub activity_repository: Arc<dyn ActivityRepository>,
    pub friendship_repository: Arc<dyn FriendshipRepository>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct GetFeedUseCaseParams {
    pub user_id: String,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl GetFeedUseCase {
    pub async fn execute(&self, params: GetFeedUseCaseParams) -> Result<Page<FeedItem>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let friend_ids = accepted_friend_ids(&*self.friendship_repository, &params.user_id).await?;

        let mut queries = JoinSet::new();
        for friend_id in &friend_ids {
            let repository = Arc::clone(&self.activity_repository);
            let friend_id = friend_id.clone();
            let cursor = params.cursor.clone();
            queries.spawn(async move {
                repository
                    .find_by_user(&friend_id, cursor.as_deref(), limit)
                    .await
            });
        }

        let mut activities = Vec::new();
        let mut any_friend_has_more = false;

        while let Some(result) = queries.join_next().await {
            let page = result.map_err(|_| Error::internal_server("Could not get feed"))??;
            any_friend_has_more |= page.len() == usize::from(limit);
            activities.extend(page);
        }

        activities.sort_by(|a, b| b.id.cmp(&a.id));
        let has_more = any_friend_has_more || activities.len() > usize::from(limit);
        activities.truncate(usize::from(limit));

        let next_cursor = has_more
            .then(|| activities.last().map(|activity| activity.id.clone()))
            .flatten();

        let author_ids: Vec<String> = activities.iter().map(|a| a.user_id.clone()).collect();
        let users: HashMap<String, User> = self
            .user_repository
            .find_by_ids(&author_ids)
            .await?
            .into_iter()
            .map(|user| (user.id.clone(), user))
            .collect();

        let items = activities
            .into_iter()
            .filter_map(|activity| {
                let user = users.get(&activity.user_id)?;
                Some(FeedItem {
                    username: user.username.clone(),
                    picture_url: user.picture_url.clone(),
                    activity,
                })
            })
            .collect();

        Ok(Page { items, next_cursor })
    }
}
