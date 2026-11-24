use async_trait::async_trait;

use crate::domain::{
    entities::ExerciseHistory,
    value_objects::{Error, Page},
};

#[async_trait]
pub trait ExerciseHistoryRepository: Send + Sync {
    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<ExerciseHistory>, Error>;
    async fn find_by_user(
        &self,
        user_id: &str,
        exercise_id: Option<&str>,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<ExerciseHistory>, Error>;
    async fn save(&self, history: &ExerciseHistory) -> Result<(), Error>;
    async fn delete(&self, user_id: &str, id: &str) -> Result<(), Error>;
    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error>;
}
