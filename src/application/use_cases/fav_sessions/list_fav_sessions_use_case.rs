use std::{collections::HashMap, sync::Arc};

use crate::{
    application::ports::repositories::{FavSessionRepository, SessionRepository},
    domain::{
        entities::Session,
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListFavSessionsUseCase {
    pub fav_session_repository: Arc<dyn FavSessionRepository>,
    pub session_repository: Arc<dyn SessionRepository>,
}

pub struct ListFavSessionsUseCaseParams {
    pub user_id: String,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListFavSessionsUseCase {
    pub async fn execute(
        &self,
        params: ListFavSessionsUseCaseParams,
    ) -> Result<Page<Session>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let favs = self
            .fav_session_repository
            .find_by_user(&params.user_id, limit, params.cursor.as_deref())
            .await?;

        let session_ids: Vec<String> = favs
            .items
            .iter()
            .map(|fav| fav.session_id.clone())
            .collect();

        let mut sessions: HashMap<String, Session> = self
            .session_repository
            .find_by_ids(&session_ids)
            .await?
            .into_iter()
            .map(|session| (session.id.clone(), session))
            .collect();

        let items = session_ids
            .iter()
            .filter_map(|id| sessions.remove(id))
            .collect();

        Ok(Page {
            items,
            next_cursor: favs.next_cursor,
        })
    }
}
