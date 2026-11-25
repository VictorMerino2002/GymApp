use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{ExerciseCategory, Muscle};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub category: ExerciseCategory,
    #[serde(alias = "muscles")]
    pub primary_muscles: Vec<Muscle>,
    #[serde(default)]
    pub secondary_muscles: Vec<Muscle>,
    pub description: String,
    pub instructions: Vec<String>,
    pub video_url: Option<String>,
    pub img_url: Option<String>,
}

impl Exercise {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        category: ExerciseCategory,
        primary_muscles: Vec<Muscle>,
        secondary_muscles: Vec<Muscle>,
        description: impl Into<String>,
        instructions: Vec<String>,
        video_url: Option<String>,
        img_url: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            name: name.into(),
            category,
            primary_muscles,
            secondary_muscles,
            description: description.into(),
            instructions,
            video_url,
            img_url,
        }
    }
}
