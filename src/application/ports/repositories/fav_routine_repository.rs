use async_trait::async_trait;

use crate::domain::{
    entities::FavRoutine,
    value_objects::{Error, Page},
};

#[async_trait]
pub trait FavRoutineRepository: Send + Sync {
    async fn exists(&self, user_id: &str, routine_id: &str) -> Result<bool, Error>;
    async fn find_by_user(
        &self,
        user_id: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<FavRoutine>, Error>;
    async fn save(&self, fav_routine: &FavRoutine) -> Result<(), Error>;
    async fn delete(&self, user_id: &str, routine_id: &str) -> Result<(), Error>;
    async fn delete_by_routine(&self, routine_id: &str) -> Result<(), Error>;
}
