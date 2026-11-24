use async_trait::async_trait;

use crate::domain::{
    entities::Session,
    value_objects::{Error, Muscle, Page},
};

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Session>, Error>;
    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
        muscle: Option<Muscle>,
    ) -> Result<Page<Session>, Error>;
    async fn find_by_author(
        &self,
        author_id: &str,
        limit: u16,
        cursor: Option<&str>,
        muscle: Option<Muscle>,
    ) -> Result<Page<Session>, Error>;
    async fn save(&self, session: &Session) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
