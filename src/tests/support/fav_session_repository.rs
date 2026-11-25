use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    application::ports::repositories::FavSessionRepository,
    domain::{
        entities::FavSession,
        value_objects::{Error, Page},
    },
};

#[derive(Default)]
pub struct InMemoryFavSessionRepository {
    favs: Mutex<Vec<FavSession>>,
}

impl InMemoryFavSessionRepository {
    pub fn all(&self) -> Vec<FavSession> {
        self.favs.lock().unwrap().clone()
    }
}

#[async_trait]
impl FavSessionRepository for InMemoryFavSessionRepository {
    async fn exists(&self, user_id: &str, session_id: &str) -> Result<bool, Error> {
        Ok(self
            .favs
            .lock()
            .unwrap()
            .iter()
            .any(|f| f.user_id == user_id && f.session_id == session_id))
    }

    async fn find_by_user(
        &self,
        _user_id: &str,
        _limit: u16,
        _cursor: Option<&str>,
    ) -> Result<Page<FavSession>, Error> {
        unimplemented!()
    }

    async fn save(&self, fav_session: &FavSession) -> Result<(), Error> {
        self.favs.lock().unwrap().push(fav_session.clone());
        Ok(())
    }

    async fn delete(&self, user_id: &str, session_id: &str) -> Result<(), Error> {
        self.favs
            .lock()
            .unwrap()
            .retain(|f| f.user_id != user_id || f.session_id != session_id);
        Ok(())
    }

    async fn delete_by_session(&self, session_id: &str) -> Result<(), Error> {
        self.favs
            .lock()
            .unwrap()
            .retain(|f| f.session_id != session_id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        self.favs.lock().unwrap().retain(|f| f.user_id != user_id);
        Ok(())
    }
}
