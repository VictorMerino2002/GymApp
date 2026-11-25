use crate::domain::{
    entities::User,
    value_objects::{Error, Page, Role},
};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), Error>;
    async fn update(&self, user: &User) -> Result<(), Error>;
    async fn delete(&self, user: &User) -> Result<(), Error>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, Error>;
    async fn increment_token_version(&self, user_id: &str) -> Result<(), Error>;
    async fn update_profile(&self, user: &User, previous_username: &str) -> Result<(), Error>;
    async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<u32, Error>;
    async fn update_progress(&self, user: &User, expected_version: u32) -> Result<bool, Error>;
    async fn update_role(&self, user_id: &str, role: Role) -> Result<(), Error>;
    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<User>, Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
    async fn search_by_username(
        &self,
        prefix: &str,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<User>, Error>;
}
