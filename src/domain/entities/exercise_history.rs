use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExerciseHistory {
    pub id: String,
    pub user_id: String,
    pub exercise_id: String,
    pub session_id: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub weight_kg: Option<f32>,
    pub reps: Option<u16>,
}
