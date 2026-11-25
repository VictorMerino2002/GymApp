use std::sync::Arc;

use crate::{
    application::{
        ports::repositories::ExerciseRepository,
        use_cases::exercises::exercise_input::{ExerciseInput, validate_exercise},
    },
    domain::{entities::Exercise, value_objects::Error},
};

pub struct CreateExerciseUseCase {
    pub exercise_repository: Arc<dyn ExerciseRepository>,
}

pub struct CreateExerciseUseCaseParams {
    pub exercise: ExerciseInput,
}

impl CreateExerciseUseCase {
    pub async fn execute(&self, params: CreateExerciseUseCaseParams) -> Result<Exercise, Error> {
        validate_exercise(&params.exercise)?;

        let input = params.exercise;
        let exercise = Exercise::new(
            input.name.trim(),
            input.category,
            input.muscles,
            input.description,
            input.instructions,
            input.video_url,
            input.img_url,
        );

        self.exercise_repository.save(&exercise).await?;
        Ok(exercise)
    }
}
