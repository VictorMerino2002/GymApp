use std::sync::Arc;

use crate::{
    application::ports::repositories::RoutineRepository,
    domain::{
        entities::Routine,
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListRoutinesUseCase {
    pub routine_repository: Arc<dyn RoutineRepository>,
}

pub struct ListRoutinesUseCaseParams {
    pub author_id: Option<String>,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListRoutinesUseCase {
    pub async fn execute(&self, params: ListRoutinesUseCaseParams) -> Result<Page<Routine>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let cursor = params.cursor.as_deref();

        match params.author_id {
            Some(author_id) => {
                self.routine_repository
                    .find_by_author(&author_id, limit, cursor)
                    .await
            }
            None => self.routine_repository.find_paginated(limit, cursor).await,
        }
    }
}
