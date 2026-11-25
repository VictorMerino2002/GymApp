use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    application::ports::repositories::FriendshipRepository,
    domain::{
        entities::{Friendship, FriendshipStatus},
        value_objects::{Error, Page},
    },
};

#[derive(Default)]
pub struct InMemoryFriendshipRepository {
    friendships: Mutex<Vec<Friendship>>,
}

impl InMemoryFriendshipRepository {
    pub fn befriend(&self, a: &str, b: &str) {
        let mut friendship = Friendship::request(a.to_string(), b.to_string()).unwrap();
        friendship.accept(b).unwrap();
        self.friendships.lock().unwrap().push(friendship);
    }

    pub fn request(&self, from: &str, to: &str) {
        let friendship = Friendship::request(from.to_string(), to.to_string()).unwrap();
        self.friendships.lock().unwrap().push(friendship);
    }

    pub fn count(&self) -> usize {
        self.friendships.lock().unwrap().len()
    }

    fn is_pair(friendship: &Friendship, user_a: &str, user_b: &str) -> bool {
        let (user1, user2) = Friendship::canonical_pair(user_a.to_string(), user_b.to_string());
        friendship.user1_id() == user1 && friendship.user2_id() == user2
    }
}

#[async_trait]
impl FriendshipRepository for InMemoryFriendshipRepository {
    async fn find(&self, user_a: &str, user_b: &str) -> Result<Option<Friendship>, Error> {
        Ok(self
            .friendships
            .lock()
            .unwrap()
            .iter()
            .find(|f| Self::is_pair(f, user_a, user_b))
            .cloned())
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        status: Option<FriendshipStatus>,
        limit: u16,
        cursor: Option<&str>,
    ) -> Result<Page<Friendship>, Error> {
        let matching: Vec<Friendship> = self
            .friendships
            .lock()
            .unwrap()
            .iter()
            .filter(|f| f.involves(user_id) && status.is_none_or(|s| f.status() == s))
            .cloned()
            .collect();

        let offset: usize = cursor.map_or(0, |c| c.parse().unwrap());
        let end = (offset + usize::from(limit)).min(matching.len());
        let next_cursor = (end < matching.len()).then(|| end.to_string());

        Ok(Page {
            items: matching[offset..end].to_vec(),
            next_cursor,
        })
    }

    async fn save(&self, friendship: &Friendship) -> Result<(), Error> {
        let mut friendships = self.friendships.lock().unwrap();
        friendships.retain(|f| !Self::is_pair(f, friendship.user1_id(), friendship.user2_id()));
        friendships.push(friendship.clone());
        Ok(())
    }

    async fn delete(&self, user_a: &str, user_b: &str) -> Result<(), Error> {
        self.friendships
            .lock()
            .unwrap()
            .retain(|f| !Self::is_pair(f, user_a, user_b));
        Ok(())
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        self.friendships
            .lock()
            .unwrap()
            .retain(|f| !f.involves(user_id));
        Ok(())
    }
}
