use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavSession {
    pub user_id: String,
    pub session_id: String,
}
