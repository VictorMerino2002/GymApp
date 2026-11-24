use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::value_objects::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Deserialize, Serialize)]
pub struct Jwt {
    pub sub: String,
    pub exp: usize,
    pub token_type: TokenType,
    #[serde(default)]
    pub role: Role,
}

impl Jwt {
    pub fn new(sub: impl Into<String>, role: Role, token_type: TokenType, hours: i64) -> Self {
        let exp = (Utc::now() + Duration::hours(hours)).timestamp() as usize;
        Self {
            sub: sub.into(),
            exp,
            token_type,
            role,
        }
    }
}
