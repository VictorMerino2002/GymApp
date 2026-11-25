use std::collections::HashMap;

use crate::domain::{
    entities::{Exercise, SessionExercise},
    value_objects::{Error, Muscle},
};

pub struct SessionExerciseInput {
    pub exercise_id: String,
    pub sets: u16,
    pub reps: u16,
    pub rest_secs: u16,
}

pub fn validate_session(name: &str, exercises: &[SessionExerciseInput]) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::bad_request("name must not be empty"));
    }

    if exercises.is_empty() {
        return Err(Error::bad_request("a session needs at least one exercise"));
    }

    if exercises.iter().any(|e| e.sets == 0 || e.reps == 0) {
        return Err(Error::bad_request("sets and reps must be greater than 0"));
    }

    Ok(())
}

pub fn exercise_ids(inputs: &[SessionExerciseInput]) -> Vec<String> {
    inputs
        .iter()
        .map(|input| input.exercise_id.clone())
        .collect()
}

pub fn build_session_exercises(
    inputs: Vec<SessionExerciseInput>,
    catalog: &[Exercise],
) -> Result<Vec<SessionExercise>, Error> {
    let catalog: HashMap<&str, &Exercise> = catalog
        .iter()
        .map(|exercise| (exercise.id.as_str(), exercise))
        .collect();

    inputs
        .into_iter()
        .map(|input| {
            let exercise = catalog.get(input.exercise_id.as_str()).ok_or_else(|| {
                Error::bad_request(&format!("Exercise {} does not exist", input.exercise_id))
            })?;

            Ok(SessionExercise {
                exercise_id: input.exercise_id,
                name: exercise.name.clone(),
                sets: input.sets,
                reps: input.reps,
                rest_secs: input.rest_secs,
            })
        })
        .collect()
}

pub fn collect_muscles(catalog: &[Exercise]) -> Vec<Muscle> {
    let mut muscles = Vec::new();

    let primary = catalog
        .iter()
        .flat_map(|exercise| &exercise.primary_muscles);
    let secondary = catalog
        .iter()
        .flat_map(|exercise| &exercise.secondary_muscles);

    for muscle in primary.chain(secondary) {
        if !muscles.contains(muscle) {
            muscles.push(*muscle);
        }
    }

    muscles
}
