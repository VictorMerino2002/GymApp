use std::collections::HashSet;

use crate::domain::value_objects::{Error, ExerciseCategory, Muscle};

pub struct ExerciseInput {
    pub name: String,
    pub category: ExerciseCategory,
    pub primary_muscles: Vec<Muscle>,
    pub secondary_muscles: Vec<Muscle>,
    pub description: String,
    pub instructions: Vec<String>,
    pub video_url: Option<String>,
    pub img_url: Option<String>,
}

pub fn validate_exercise(input: &ExerciseInput) -> Result<(), Error> {
    if input.name.trim().is_empty() {
        return Err(Error::bad_request("name must not be empty"));
    }

    if input.primary_muscles.is_empty() {
        return Err(Error::bad_request(
            "an exercise needs at least one primary muscle",
        ));
    }

    let mut seen = HashSet::new();
    let all_muscles = input.primary_muscles.iter().chain(&input.secondary_muscles);

    if !all_muscles.into_iter().all(|muscle| seen.insert(muscle)) {
        return Err(Error::bad_request(
            "a muscle can only appear once across primary_muscles and secondary_muscles",
        ));
    }

    Ok(())
}
