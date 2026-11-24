use std::sync::Arc;

use serde::Serialize;

use crate::{
    application::ports::repositories::UserRepository,
    domain::value_objects::{Error, Page},
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub picture_url: Option<String>,
}

pub struct ListUsersUseCase {
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct ListUsersUseCaseParams {
    pub username: String,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListUsersUseCase {
    pub async fn execute(
        &self,
        params: ListUsersUseCaseParams,
    ) -> Result<Page<UserSummary>, Error> {
        let username = params.username.trim();

        if username.is_empty() {
            return Err(Error::bad_request("username must not be empty"));
        }

        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let users = self
            .user_repository
            .search_by_username(username, limit, params.cursor.as_deref())
            .await?;

        let items = users
            .items
            .into_iter()
            .map(|user| UserSummary {
                id: user.id,
                username: user.username,
                picture_url: user.picture_url,
            })
            .collect();

        Ok(Page {
            items,
            next_cursor: users.next_cursor,
        })
    }
}
