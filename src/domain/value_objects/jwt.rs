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
    #[serde(default)]
    pub token_version: u32,
}

impl Jwt {
    pub fn new(
        sub: impl Into<String>,
        role: Role,
        token_version: u32,
        token_type: TokenType,
        ttl: Duration,
    ) -> Self {
        let exp = (Utc::now() + ttl).timestamp() as usize;
        Self {
            sub: sub.into(),
            exp,
            token_type,
            role,
            token_version,
        }
    }
}
