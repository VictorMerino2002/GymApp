use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::{Timestamp, Uuid};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    #[serde(flatten)]
    pub kind: ActivityKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActivityKind {
    SessionCompleted {
        session_id: String,
        session_name: String,
    },
    PersonalRecord {
        exercise_id: String,
        exercise_name: String,
        weight_kg: f32,
        reps: u16,
    },
    LevelUp {
        level: u32,
    },
}

impl Activity {
    pub fn new(user_id: impl Into<String>, kind: ActivityKind) -> Self {
        let created_at = Utc::now();
        let timestamp = Timestamp::from_unix(
            uuid::NoContext,
            created_at.timestamp().max(0) as u64,
            created_at.timestamp_subsec_nanos(),
        );

        Self {
            id: Uuid::new_v7(timestamp).into(),
            user_id: user_id.into(),
            created_at,
            kind,
        }
    }
}
