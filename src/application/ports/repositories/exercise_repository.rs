use async_trait::async_trait;

use crate::domain::{
    entities::Exercise,
    value_objects::{Equipment, Error, ExerciseCategory, Muscle, Page},
};

#[async_trait]
pub trait ExerciseRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<Exercise>, Error>;
    async fn find_by_ids(&self, ids: &[String]) -> Result<Vec<Exercise>, Error>;
    async fn find_all(&self) -> Result<Vec<Exercise>, Error>;
    async fn find_paginated(
        &self,
        limit: u16,
        cursor: Option<&str>,
        muscle: Option<Muscle>,
        category: Option<ExerciseCategory>,
        equipment: Option<Equipment>,
    ) -> Result<Page<Exercise>, Error>;
    async fn save(&self, exercise: &Exercise) -> Result<(), Error>;
    async fn delete(&self, id: &str) -> Result<(), Error>;
}
