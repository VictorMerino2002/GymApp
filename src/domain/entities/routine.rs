use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{Author, DifficultyLevel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Routine {
    pub id: String,
    pub author: Author,
    pub title: String,
    pub level: DifficultyLevel,
    pub duration_weeks: u16,
    pub description: String,
    pub sessions: Vec<SessionRef>,
    pub created_at: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRef {
    pub session_id: String,
    pub name: String,
}

impl Routine {
    pub fn new(
        title: impl Into<String>,
        author: Author,
        level: DifficultyLevel,
        duration_weeks: u16,
        description: impl Into<String>,
        sessions: Vec<SessionRef>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            author,
            title: title.into(),
            level,
            duration_weeks,
            description: description.into(),
            sessions,
            created_at: Utc::now().date_naive(),
        }
    }
}
