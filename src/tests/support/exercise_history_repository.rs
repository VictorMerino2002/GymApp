use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    application::ports::repositories::ExerciseHistoryRepository,
    domain::{
        entities::ExerciseHistory,
        value_objects::{Error, Page},
    },
};

#[derive(Default)]
pub struct InMemoryExerciseHistoryRepository {
    entries: Mutex<Vec<ExerciseHistory>>,
}

impl InMemoryExerciseHistoryRepository {
    pub fn all(&self) -> Vec<ExerciseHistory> {
        self.entries.lock().unwrap().clone()
    }
}

#[async_trait]
impl ExerciseHistoryRepository for InMemoryExerciseHistoryRepository {
    async fn find_by_id(&self, user_id: &str, id: &str) -> Result<Option<ExerciseHistory>, Error> {
        Ok(self
            .entries
            .lock()
            .unwrap()
            .iter()
            .find(|e| e.user_id == user_id && e.id == id)
            .cloned())
    }

    async fn find_by_user(
        &self,
        user_id: &str,
        exercise_id: Option<&str>,
        limit: u16,
        _cursor: Option<&str>,
    ) -> Result<Page<ExerciseHistory>, Error> {
        let mut items: Vec<ExerciseHistory> = self
            .entries
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.user_id == user_id && exercise_id.is_none_or(|id| e.exercise_id == id))
            .cloned()
            .collect();
        items.truncate(usize::from(limit));
        Ok(Page {
            items,
            next_cursor: None,
        })
    }

    async fn save(&self, history: &ExerciseHistory) -> Result<(), Error> {
        self.entries.lock().unwrap().push(history.clone());
        Ok(())
    }

    async fn delete(&self, user_id: &str, id: &str) -> Result<(), Error> {
        self.entries
            .lock()
            .unwrap()
            .retain(|e| e.user_id != user_id || e.id != id);
        Ok(())
    }

    async fn delete_by_user(&self, user_id: &str) -> Result<(), Error> {
        self.entries
            .lock()
            .unwrap()
            .retain(|e| e.user_id != user_id);
        Ok(())
    }
}
