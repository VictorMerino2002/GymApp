use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    application::ports::repositories::FavRoutineRepository,
    domain::{
        entities::FavRoutine,
        value_objects::{Error, Page},
    },
};

#[derive(Default)]
pub struct InMemoryFavRoutineRepository {
    favs: Mutex<Vec<FavRoutine>>,
}

impl InMemoryFavRoutineRepository {
    pub fn all(&self) -> Vec<FavRoutine> {
        self.favs.lock().unwrap().clone()
    }
}

#[async_trait]
impl FavRoutineRepository for InMemoryFavRoutineRepository {
    async fn exists(&self, user_id: &str, routine_id: &str) -> Result<bool, Error> {
        Ok(self
            .favs
            .lock()
            .unwrap()
            .iter()
            .any(|f| f.user_id == user_id && f.routine_id == routine_id))
    }

    async fn find_by_user(
        &self,
        _user_id: &str,
        _limit: u16,
        _cursor: Option<&str>,
    ) -> Result<Page<FavRoutine>, Error> {
        unimplemented!()
    }

    async fn save(&self, fav_routine: &FavRoutine) -> Result<(), Error> {
        self.favs.lock().unwrap().push(fav_routine.clone());
        Ok(())
    }

    async fn delete(&self, user_id: &str, routine_id: &str) -> Result<(), Error> {
        self.favs
            .lock()
            .unwrap()
            .retain(|f| f.user_id != user_id || f.routine_id != routine_id);
        Ok(())
    }

    async fn delete_by_routine(&self, routine_id: &str) -> Result<(), Error> {
        self.favs
            .lock()
            .unwrap()
            .retain(|f| f.routine_id != routine_id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        self.favs.lock().unwrap().retain(|f| f.user_id != user_id);
        Ok(())
    }
}
