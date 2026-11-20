use serde::{Deserialize, Serialize};

use crate::domain::value_objects::Author;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub author: Author,
    pub exercises: Vec<SessionExercise>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionExercise {
    pub exercise_id: String,
    pub name: String,
    pub sets: u16,
    pub reps: u16,
    pub rest_secs: u16,
}
