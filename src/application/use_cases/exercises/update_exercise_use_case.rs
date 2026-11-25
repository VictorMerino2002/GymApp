use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::ExerciseRepository,
        use_cases::exercises::exercise_input::{ExerciseInput, validate_exercise},
    },
    domain::{entities::Exercise, value_objects::Error},
};

pub struct UpdateExerciseUseCase {
    pub exercise_repository: Arc<dyn ExerciseRepository>,
}

pub struct UpdateExerciseUseCaseParams {
    pub id: String,
    pub exercise: ExerciseInput,
}

impl UpdateExerciseUseCase {
    pub async fn execute(&self, params: UpdateExerciseUseCaseParams) -> Result<Exercise, Error> {
        validate_exercise(&params.exercise)?;

        let mut exercise = self
            .exercise_repository
            .find_by_id(&params.id)
            .await?
            .ok_or_else(|| Error::not_found("Exercise not found"))?;

        let input = params.exercise;
        exercise.name = input.name.trim().to_string();
        exercise.category = input.category;
        exercise.primary_muscles = input.primary_muscles;
        exercise.secondary_muscles = input.secondary_muscles;
        exercise.description = input.description;
        exercise.instructions = input.instructions;
        exercise.video_url = input.video_url;
        exercise.img_url = input.img_url;

        self.exercise_repository.save(&exercise).await?;
        Ok(exercise)
    }
}
