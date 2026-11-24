use async_trait::async_trait;

use crate::domain::{
    entities::Routine,
    value_objects::{Error, Page},
};

#[async_trait]
pub trait RoutineRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Routine>, Error>;
    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Routine>, Error>;
    async fn find_by_author(
        &self,
        author_id: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Routine>, Error>;
    async fn save(&self, routine: &Routine) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
