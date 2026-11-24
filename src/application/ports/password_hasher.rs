use async_trait::async_trait;

use crate::domain::value_objects::Error;

#[async_trait]
pub trait PasswordHasherPort: Send + Sync {
    async fn hash(&self, password: &str) -> Result<String, Error>;
    async fn verify(&self, password: &str, hash: &str) -> Result<bool, Error>;
}
