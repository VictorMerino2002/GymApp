use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::value_objects::{Error, Role};

const MIN_USERNAME_LENGTH: usize = 3;
const MAX_USERNAME_LENGTH: usize = 20;
const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;
const MAX_PICTURE_URL_LENGTH: usize = 2048;
const XP_PER_SET: u32 = 10;
const XP_PER_SESSION: u32 = 50;
const XP_PERSONAL_RECORD: u32 = 25;
const XP_PER_STREAK_WEEK: u32 = 5;
const MAX_STREAK_BONUS: u32 = 50;
const DAILY_XP_CAP: u32 = 300;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub picture_url: Option<String>,
    pub stats: Stats,
    #[serde(default)]
    pub role: Role,
    #[serde(default)]
    pub personal_records: HashMap<String, PersonalRecord>,
    #[serde(default)]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub token_version: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub xp: u32,
    pub sessions_done: u32,
    pub exercises_done: u32,
    #[serde(default)]
    pub streak_weeks: u32,
    #[serde(default)]
    pub last_workout_date: Option<NaiveDate>,
    #[serde(default)]
    pub xp_today: u32,
    #[serde(default)]
    pub progress_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PersonalRecord {
    pub weight_kg: f32,
    pub reps: u16,
}

impl PersonalRecord {
    fn beats(&self, other: &PersonalRecord) -> bool {
        self.weight_kg > other.weight_kg
            || (self.weight_kg == other.weight_kg && self.reps > other.reps)
    }
}

impl Stats {
    pub fn level(&self) -> u32 {
        level_from_xp(self.xp)
    }
}

pub fn validate_username(username: &str) -> Result<(), Error> {
    let length = username.chars().count();

    if !(MIN_USERNAME_LENGTH..=MAX_USERNAME_LENGTH).contains(&length) {
        return Err(Error::bad_request(&format!(
            "username must be between {MIN_USERNAME_LENGTH} and {MAX_USERNAME_LENGTH} characters"
        )));
    }

    if !username
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    {
        return Err(Error::bad_request(
            "username can only contain letters, numbers, '_' and '.'",
        ));
    }

    if !username.starts_with(|c: char| c.is_ascii_alphanumeric()) {
        return Err(Error::bad_request(
            "username must start with a letter or a number",
        ));
    }

    Ok(())
}

pub fn validate_password(password: &str) -> Result<(), Error> {
    let length = password.chars().count();

    if !(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).contains(&length) {
        return Err(Error::bad_request(&format!(
            "password must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH} characters"
        )));
    }

    Ok(())
}

pub fn validate_picture_url(url: &str) -> Result<(), Error> {
    if url.len() > MAX_PICTURE_URL_LENGTH
        || !url.starts_with("https://")
        || url.chars().any(char::is_whitespace)
    {
        return Err(Error::bad_request("picture_url must be a valid https URL"));
    }

    Ok(())
}

pub fn level_from_xp(xp: u32) -> u32 {
    (((1.0 + f64::from(xp) / 12.5).sqrt() - 1.0) / 2.0).floor() as u32
}

fn week_start(date: NaiveDate) -> NaiveDate {
    date - Duration::days(i64::from(date.weekday().num_days_from_monday()))
}

pub fn xp_for_level(level: u32) -> u32 {
    50 * level * (level + 1)
}

impl User {
    pub fn new(username: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().into(),
            username: username.into(),
            picture_url: None,
            stats: Stats::default(),
            role: Role::User,
            personal_records: HashMap::new(),
            password_hash: Some(password_hash.into()),
            token_version: 0,
        }
    }

    pub fn record_exercise(
        &mut self,
        exercise_id: &str,
        weight_kg: Option<f32>,
        reps: Option<u16>,
        today: NaiveDate,
    ) -> u32 {
        let mut xp = XP_PER_SET + self.register_workout_day(today);

        if self.update_personal_record(exercise_id, weight_kg, reps) {
            xp += XP_PERSONAL_RECORD;
        }

        self.stats.exercises_done += 1;
        self.award_xp(xp)
    }

    pub fn complete_session(&mut self, today: NaiveDate) -> u32 {
        let xp = XP_PER_SESSION + self.register_workout_day(today);

        self.stats.sessions_done += 1;
        self.award_xp(xp)
    }

    fn register_workout_day(&mut self, today: NaiveDate) -> u32 {
        let stats = &mut self.stats;

        if stats.last_workout_date == Some(today) {
            return 0;
        }

        let this_week = week_start(today);

        stats.streak_weeks = match stats.last_workout_date.map(week_start) {
            Some(last_week) if last_week == this_week => stats.streak_weeks.max(1),
            Some(last_week) if last_week == this_week - Duration::weeks(1) => {
                stats.streak_weeks + 1
            }
            _ => 1,
        };
        stats.last_workout_date = Some(today);
        stats.xp_today = 0;

        (XP_PER_STREAK_WEEK * (stats.streak_weeks - 1)).min(MAX_STREAK_BONUS)
    }

    fn award_xp(&mut self, xp: u32) -> u32 {
        let stats = &mut self.stats;
        let awarded = xp.min(DAILY_XP_CAP.saturating_sub(stats.xp_today));

        stats.xp_today += awarded;
        stats.xp += awarded;

        awarded
    }

    pub fn undo_exercise(&mut self, xp_awarded: u32) {
        self.stats.exercises_done = self.stats.exercises_done.saturating_sub(1);
        self.stats.xp = self.stats.xp.saturating_sub(xp_awarded);
    }

    fn update_personal_record(
        &mut self,
        exercise_id: &str,
        weight_kg: Option<f32>,
        reps: Option<u16>,
    ) -> bool {
        let Some(reps) = reps else {
            return false;
        };

        let candidate = PersonalRecord {
            weight_kg: weight_kg.unwrap_or(0.0),
            reps,
        };

        match self.personal_records.get(exercise_id) {
            None => {
                self.personal_records
                    .insert(exercise_id.to_string(), candidate);
                false
            }
            Some(record) if candidate.beats(record) => {
                self.personal_records
                    .insert(exercise_id.to_string(), candidate);
                true
            }
            Some(_) => false,
        }
    }
}
