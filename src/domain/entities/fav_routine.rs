use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FavRoutine {
    pub user_id: String,
    pub routine_id: String,
}
