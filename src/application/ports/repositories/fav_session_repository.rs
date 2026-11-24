use async_trait::async_trait;

use crate::domain::{
    entities::FavSession,
    value_objects::{Error, Page},
};

#[async_trait]
pub trait FavSessionRepository: Send + Sync {
    async fn exists(&self, user_id: &str, session_id: &str) -> Result<bool, Error>;
    async fn find_by_user(
        &self,
        user_id: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<FavSession>, Error>;
    async fn save(&self, fav_session: &FavSession) -> Result<(), Error>;
    async fn delete(&self, user_id: &str, session_id: &str) -> Result<(), Error>;
    async fn delete_by_session(&self, session_id: &str) -> Result<(), Error>;
    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error>;
}
