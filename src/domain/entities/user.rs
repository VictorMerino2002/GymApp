use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub picture_url: Option<String>,
    pub stats: Stats,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub level: u32,
    pub sessions_done: u32,
    pub exercises_done: u32,
}

impl Stats {
    pub fn default() -> Self {
        Self {
            level: 0,
            sessions_done: 0,
            exercises_done: 0,
        }
    }
}

impl User {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            username: username.into(),
            picture_url: None,
            stats: Stats::default(),
        }
    }
}
