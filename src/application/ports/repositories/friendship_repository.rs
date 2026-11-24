use async_trait::async_trait;

use crate::domain::{
    entities::{Friendship, FriendshipStatus},
    value_objects::{Error, Page},
};

#[async_trait]
pub trait FriendshipRepository: Send + Sync {
    async fn find(&self, user_a: &str, user_b: &str) -> Result<Option<Friendship>, Error>;
    async fn find_by_user(
        &self,
        user_id: &str,
        status: Option<FriendshipStatus>,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Friendship>, Error>;
    async fn save(&self, friendship: &Friendship) -> Result<(), Error>;
    async fn delete(&self, user_a: &str, user_b: &str) -> Result<(), Error>;
}
