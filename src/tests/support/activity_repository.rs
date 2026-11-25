use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    application::ports::repositories::ActivityRepository,
    domain::{entities::Activity, value_objects::Error},
};

#[derive(Default)]
pub struct InMemoryActivityRepository {
    activities: Mutex<Vec<Activity>>,
}

impl InMemoryActivityRepository {
    pub fn all(&self) -> Vec<Activity> {
        self.activities.lock().unwrap().clone()
    }
}

#[async_trait]
impl ActivityRepository for InMemoryActivityRepository {
    async fn save(&self, activity: &Activity) -> Result<(), Error> {
        self.activities.lock().unwrap().push(activity.clone());
        Ok(())
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        before: Option<&str>,
        limit: u16,
    ) -> Result<Vec<Activity>, Error> {
        let mut activities: Vec<Activity> = self
            .activities
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.user_id == user_id && before.is_none_or(|b| a.id.as_str() < b))
            .cloned()
            .collect();
        activities.sort_by(|a, b| b.id.cmp(&a.id));
        activities.truncate(usize::from(limit));
        Ok(activities)
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        self.activities
            .lock()
            .unwrap()
            .retain(|a| a.user_id != user_id);
        Ok(())
    }
}
