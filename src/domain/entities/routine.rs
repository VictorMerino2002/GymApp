use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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
