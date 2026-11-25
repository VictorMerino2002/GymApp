use std::{collections::BTreeMap, sync::Mutex};

use async_trait::async_trait;

use crate::{
    application::ports::repositories::ExerciseRepository,
    domain::{
        entities::Exercise,
        value_objects::{Error, Muscle, Page},
    },
};

#[derive(Default)]
pub struct InMemoryExerciseRepository {
    exercises: Mutex<BTreeMap<String, Exercise>>,
}

impl InMemoryExerciseRepository {
    pub fn with(exercises: Vec<Exercise>) -> Self {
        let repository = Self::default();
        for exercise in exercises {
            repository.insert(exercise);
        }
        repository
    }

    pub fn insert(&self, exercise: Exercise) {
        self.exercises
            .lock()
            .unwrap()
            .insert(exercise.id.clone(), exercise);
    }

    pub fn get(&self, id: &str) -> Option<Exercise> {
        self.exercises.lock().unwrap().get(id).cloned()
    }
}

#[async_trait]
impl ExerciseRepository for InMemoryExerciseRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Exercise>, Error> {
        Ok(self.get(id))
    }

    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<Exercise>, Error> {
        let exercises = self.exercises.lock().unwrap();
        Ok(ids
            .iter()
            .filter_map(|id| exercises.get(id).cloned())
            .collect())
    }

    async fn find_all(&self) -> Result<Vec<Exercise>, Error> {
        Ok(self.exercises.lock().unwrap().values().cloned().collect())
    }

    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
        muscle: Option<Muscle>,
    ) -> Result<Page<Exercise>, Error> {
        let matching: Vec<Exercise> = self
            .exercises
            .lock()
            .unwrap()
            .values()
            .filter(|e| cursor.is_none_or(|c| e.id.as_str() > c))
            .filter(|e| muscle.is_none_or(|m| e.muscles.contains(&m)))
            .cloned()
            .collect();

        let has_more = matching.len() > usize::from(limit);
        let items: Vec<Exercise> = matching.into_iter().take(usize::from(limit)).collect();
        let next_cursor = has_more
            .then(|| items.last().map(|e| e.id.clone()))
            .flatten();

        Ok(Page { items, next_cursor })
    }

    async fn save(&self, exercise: &Exercise) -> Result<(), Error> {
        self.insert(exercise.clone());
        Ok(())
    }

    async fn delete(&self, id: &str) -> Result<(), Error> {
        self.exercises.lock().unwrap().remove(id);
        Ok(())
    }
}
