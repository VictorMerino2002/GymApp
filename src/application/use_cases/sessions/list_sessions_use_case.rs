use std::sync::Arc;

use crate::{
    application::ports::repositories::SessionRepository,
    domain::{
        entities::Session,
        value_objects::{Error, Muscle, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListSessionsUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct ListSessionsUseCaseParams {
    pub author_id: Option<String>,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
    pub muscle: Option<Muscle>,
}

impl ListSessionsUseCase {
    pub async fn execute(&self, params: ListSessionsUseCaseParams) -> Result<Page<Session>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let cursor = params.cursor.as_deref();

        match params.author_id {
            Some(author_id) => {
                self.session_repository
                    .find_by_author(&author_id, limit, cursor, params.muscle)
                    .await
            }
            None => {
                self.session_repository
                    .find_paginated(limit, cursor, params.muscle)
                    .await
            }
        }
    }
}
