use async_trait::async_trait;

use crate::domain::{entities::Activity, value_objects::Error};

#[async_trait]
pub trait ActivityRepository: Send + Sync {
    async fn save(&self, activity: &Activity) -> Result<(), Error>;
    async fn find_by_user(
        &self,
        user_id: &str,
        before: Option<&str>,
        limit: u16,
    ) -> Result<Vec<Activity>, Error>;
    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error>;
}
