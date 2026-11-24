use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{Author, Muscle};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub author: Author,
    pub exercises: Vec<SessionExercise>,
    #[serde(default)]
    pub muscles: Vec<Muscle>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionExercise {
    pub exercise_id: String,
    pub name: String,
    pub sets: u16,
    pub reps: u16,
    pub rest_secs: u16,
}

impl Session {
    pub fn new(
        name: impl Into<String>,
        author: Author,
        exercises: Vec<SessionExercise>,
        muscles: Vec<Muscle>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            name: name.into(),
            author,
            exercises,
            muscles,
        }
    }
}
