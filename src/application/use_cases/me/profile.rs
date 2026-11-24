use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{
    entities::{PersonalRecord, User, xp_for_level},
    value_objects::Role,
};

#[derive(Debug, Serialize)]
pub struct Profile {
    pub id: String,
    pub username: String,
    pub picture_url: Option<String>,
    pub role: Role,
    pub stats: ProfileStats,
    pub personal_records: HashMap<String, PersonalRecord>,
}

#[derive(Debug, Serialize)]
pub struct ProfileStats {
    pub level: u32,
    pub xp: u32,
    pub current_level_xp: u32,
    pub next_level_xp: u32,
    pub exercises_done: u32,
    pub sessions_done: u32,
    pub streak_weeks: u32,
}

impl From<User> for Profile {
    fn from(user: User) -> Self {
        let level = user.stats.level();

        Self {
            id: user.id,
            username: user.username,
            picture_url: user.picture_url,
            role: user.role,
            stats: ProfileStats {
                level,
                xp: user.stats.xp,
                current_level_xp: xp_for_level(level),
                next_level_xp: xp_for_level(level + 1),
                exercises_done: user.stats.exercises_done,
                sessions_done: user.stats.sessions_done,
                streak_weeks: user.stats.streak_weeks,
            },
            personal_records: user.personal_records,
        }
    }
}
