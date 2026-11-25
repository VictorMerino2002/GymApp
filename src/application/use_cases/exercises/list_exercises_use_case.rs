use std::sync::Arc;

use crate::{
    application::ports::repositories::ExerciseRepository,
    domain::{
        entities::Exercise,
        value_objects::{Equipment, Error, ExerciseCategory, Muscle, Page},
    },
};

const DEFAULT_LIMIT: u16 = 20;
const MAX_LIMIT: u16 = 100;

pub struct ListExercisesUseCase {
    pub exercise_repository: Arc<dyn ExerciseRepository>,
}

pub struct ListExercisesUseCaseParams {
    pub limit: Option<u16>,
    pub cursor: Option<String>,
    pub muscle: Option<Muscle>,
    pub category: Option<ExerciseCategory>,
    pub equipment: Option<Equipment>,
}

impl ListExercisesUseCase {
    pub async fn execute(
        &self,
        params: ListExercisesUseCaseParams,
    ) -> Result<Page<Exercise>, Error> {
        let limit = params.limit.unwrap_or(DEFAULT_LIMIT);

        if !(1..=MAX_LIMIT).contains(&limit) {
            return Err(Error::bad_request(&format!(
                "limit must be between 1 and {MAX_LIMIT}"
            )));
        }

        self.exercise_repository
            .find_paginated(
                limit,
                params.cursor.as_deref(),
                params.muscle,
                params.category,
                params.equipment,
            )
            .await
    }
}
