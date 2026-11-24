use std::{collections::HashMap, sync::Arc};

use crate::{
    application::ports::repositories::{FavRoutineRepository, RoutineRepository},
    domain::{
        entities::Routine,
        value_objects::{Error, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListFavRoutinesUseCase {
    pub fav_routine_repository: Arc<dyn FavRoutineRepository>,
    pub routine_repository: Arc<dyn RoutineRepository>,
}

pub struct ListFavRoutinesUseCaseParams {
    pub user_id: String,
    pub limit: Option<u16>,
    pub cursor: Option<String>,
}

impl ListFavRoutinesUseCase {
    pub async fn execute(
        &self,
        params: ListFavRoutinesUseCaseParams,
    ) -> Result<Page<Routine>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        let favs = self
            .fav_routine_repository
            .find_by_user(&params.user_id, limit, params.cursor.as_deref())
            .await?;

        let routine_ids: Vec<String> = favs
            .items
            .iter()
            .map(|fav| fav.routine_id.clone())
            .collect();

        let mut routines: HashMap<String, Routine> = self
            .routine_repository
            .find_by_ids(&routine_ids)
            .await?
            .into_iter()
            .map(|routine| (routine.id.clone(), routine))
            .collect();

        let items = routine_ids
            .iter()
            .filter_map(|id| routines.remove(id))
            .collect();

        Ok(Page {
            items,
            next_cursor: favs.next_cursor,
        })
    }
}
