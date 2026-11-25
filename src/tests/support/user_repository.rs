use std::{
    collections::HashMap,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use async_trait::async_trait;

use crate::{
    application::ports::repositories::UserRepository,
    domain::{
        entities::User,
        value_objects::{Error, Page, Role},
    },
};

#[derive(Default)]
pub struct InMemoryUserRepository {
    users: Mutex<HashMap<String, User>>,
    pub progress_conflicts: AtomicU32,
}

impl InMemoryUserRepository {
    pub fn with(users: Vec<User>) -> Self {
        let repository = Self::default();
        for user in users {
            repository.insert(user);
        }
        repository
    }

    pub fn insert(&self, user: User) {
        self.users.lock().unwrap().insert(user.id.clone(), user);
    }

    pub fn get(&self, id: &str) -> Option<User> {
        self.users.lock().unwrap().get(id).cloned()
    }

    pub fn remove(&self, id: &str) {
        self.users.lock().unwrap().remove(id);
    }

    fn username_taken(&self, username: &str, except_id: &str) -> bool {
        let username = username.to_lowercase();
        self.users
            .lock()
            .unwrap()
            .values()
            .any(|user| user.id != except_id && user.username.to_lowercase() == username)
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: &User) -> Result<(), Error> {
        if self.username_taken(&user.username, &user.id) {
            return Err(Error::conflict("Username already taken"));
        }
        self.insert(user.clone());
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), Error> {
        self.insert(user.clone());
        Ok(())
    }

    async fn delete(&self, user: &User) -> Result<(), Error> {
        self.remove(&user.id);
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>, Error> {
        Ok(self.get(id))
    }

    async fn increment_token_version(&self, user_id: &str) -> Result<(), Error> {
        let mut users = self.users.lock().unwrap();
        let user = users
            .get_mut(user_id)
            .ok_or_else(|| Error::unauthorized("User not found"))?;
        user.token_version += 1;
        Ok(())
    }

    async fn update_profile(&self, user: &User, _previous_username: &str) -> Result<(), Error> {
        if self.username_taken(&user.username, &user.id) {
            return Err(Error::conflict("Username already taken"));
        }
        let mut users = self.users.lock().unwrap();
        let stored = users
            .get_mut(&user.id)
            .ok_or_else(|| Error::not_found("User not found"))?;
        stored.username = user.username.clone();
        stored.picture_url = user.picture_url.clone();
        Ok(())
    }

    async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<u32, Error> {
        let mut users = self.users.lock().unwrap();
        let user = users
            .get_mut(user_id)
            .ok_or_else(|| Error::unauthorized("User not found"))?;
        user.password_hash = Some(password_hash.to_string());
        user.token_version += 1;
        Ok(user.token_version)
    }

    async fn update_progress(&self, user: &User, expected_version: u32) -> Result<bool, Error> {
        if self
            .progress_conflicts
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1))
            .is_ok()
        {
            return Ok(false);
        }

        let mut users = self.users.lock().unwrap();
        let Some(stored) = users.get_mut(&user.id) else {
            return Ok(false);
        };
        if stored.stats.progress_version != expected_version {
            return Ok(false);
        }
        stored.stats = user.stats;
        stored.personal_records = user.personal_records.clone();
        Ok(true)
    }

    async fn update_role(&self, user_id: &str, role: Role) -> Result<(), Error> {
        let mut users = self.users.lock().unwrap();
        let user = users
            .get_mut(user_id)
            .ok_or_else(|| Error::not_found("User not found"))?;
        user.role = role;
        Ok(())
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<User>, Error> {
        let users = self.users.lock().unwrap();
        Ok(ids.iter().filter_map(|id| users.get(id).cloned()).collect())
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let username = username.to_lowercase();
        Ok(self
            .users
            .lock()
            .unwrap()
            .values()
            .find(|user| user.username.to_lowercase() == username)
            .cloned())
    }

    async fn search_by_username(
        &self,
        prefix: &str,
        limit: u16,
        _cursor: Option<&str>,
    ) -> Result<Page<User>, Error> {
        let prefix = prefix.to_lowercase();
        let mut items: Vec<User> = self
            .users
            .lock()
            .unwrap()
            .values()
            .filter(|user| user.username.to_lowercase().starts_with(&prefix))
            .cloned()
            .collect();
        items.sort_by_key(|user| user.username.to_lowercase());
        items.truncate(usize::from(limit));
        Ok(Page {
            items,
            next_cursor: None,
        })
    }
}
