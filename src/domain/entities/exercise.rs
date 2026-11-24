use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::Muscle;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub muscles: Vec<Muscle>,
    pub description: String,
    pub instructions: Vec<String>,
    pub video_url: Option<String>,
    pub img_url: Option<String>,
}

impl Exercise {
    pub fn new(
        name: impl Into<String>,
        muscles: Vec<Muscle>,
        description: impl Into<String>,
        instructions: Vec<String>,
        video_url: Option<String>,
        img_url: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            name: name.into(),
            muscles,
            description: description.into(),
            instructions,
            video_url,
            img_url,
        }
    }
}
