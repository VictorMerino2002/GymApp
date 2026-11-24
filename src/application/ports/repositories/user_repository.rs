use crate::domain::{entities::User, value_objects::Error};
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn save(&self, user: &User) -> Result<(), Error>;
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error>;
}
