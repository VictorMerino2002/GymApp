use crate::domain::value_objects::{Error, Muscle};

pub struct ExerciseInput {
    pub name: String,
    pub muscles: Vec<Muscle>,
    pub description: String,
    pub instructions: Vec<String>,
    pub video_url: Option<String>,
    pub img_url: Option<String>,
}

pub fn validate_exercise(input: &ExerciseInput) -> Result<(), Error> {
    if input.name.trim().is_empty() {
        return Err(Error::bad_request("name must not be empty"));
    }

    if input.muscles.is_empty() {
        return Err(Error::bad_request("an exercise needs at least one muscle"));
    }

    Ok(())
}
