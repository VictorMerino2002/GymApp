use std::sync::Arc;

use chrono::Duration;

use crate::{
    application::ports::JwtEncoderPort,
    domain::value_objects::{Error, Jwt, Role, TokenType},
};

const ACCESS_TOKEN_MINUTES: i64 = 15;
const REFRESH_TOKEN_HOURS: i64 = 72;

pub struct GenerateAuthUseCase {
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
}

pub struct GenerateAuthUseCaseParams {
    pub user_id: String,
    pub role: Role,
    pub token_version: u32,
}

pub struct GenerateAuthUseCaseResponse {
    pub access: String,
    pub refresh: String,
}

impl GenerateAuthUseCase {
    pub fn execute(
        &self,
        params: &GenerateAuthUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        let access_jwt = Jwt::new(
            &params.user_id,
            params.role,
            params.token_version,
            TokenType::Access,
            Duration::minutes(ACCESS_TOKEN_MINUTES),
        );
        let refresh_jwt = Jwt::new(
            &params.user_id,
            params.role,
            params.token_version,
            TokenType::Refresh,
            Duration::hours(REFRESH_TOKEN_HOURS),
        );

        let access_tkn = self.jwt_encoder.encode(&access_jwt)?;
        let refresh_tkn = self.jwt_encoder.encode(&refresh_jwt)?;

        Ok(GenerateAuthUseCaseResponse {
            access: access_tkn,
            refresh: refresh_tkn,
        })
    }
}
