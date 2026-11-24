use std::sync::Arc;

use crate::{
    application::ports::repositories::SessionRepository,
    domain::{
        entities::Session,
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListSessionsUseCase {
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct ListSessionsUseCaseParams {
    pub author_id: String,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListSessionsUseCase {
    pub async fn execute(&self, params: ListSessionsUseCaseParams) -> Result<Page<Session>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        self.session_repository
            .find_by_author(&params.author_id, limit, params.cursor.as_deref())
            .await
    }
}
