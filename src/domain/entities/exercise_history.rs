use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::{Timestamp, Uuid};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExerciseHistory {
    pub id: String,
    pub user_id: String,
    pub exercise_id: String,
    pub session_id: Option<String>,
    pub performed_at: DateTime<Utc>,
    pub weight_kg: Option<f32>,
    pub reps: Option<u16>,
    #[serde(default)]
    pub xp_awarded: u32,
}

impl ExerciseHistory {
    pub fn new(
        user_id: impl Into<String>,
        exercise_id: impl Into<String>,
        session_id: Option<String>,
        performed_at: DateTime<Utc>,
        weight_kg: Option<f32>,
        reps: Option<u16>,
    ) -> Self {
        let timestamp = Timestamp::from_unix(
            uuid::NoContext,
            performed_at.timestamp().max(0) as u64,
            performed_at.timestamp_subsec_nanos(),
        );

        Self {
            id: Uuid::new_v7(timestamp).into(),
            user_id: user_id.into(),
            exercise_id: exercise_id.into(),
            session_id,
            performed_at,
            weight_kg,
            reps,
            xp_awarded: 0,
        }
    }
}
