use std::sync::Arc;

use crate::{
    application::{
        ports::{JwtEncoderPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::value_objects::{Error, TokenType},
};

pub struct RefreshTokenUseCase {
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct RefreshTokenUseCaseParams {
    pub refresh_token: String,
}

impl RefreshTokenUseCase {
    pub async fn execute(
        &self,
        params: RefreshTokenUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        let jwt = self
            .jwt_encoder
            .decode(&params.refresh_token)
            .map_err(|_| Error::unauthorized("Invalid refresh token"))?;

        if jwt.token_type != TokenType::Refresh {
            return Err(Error::unauthorized("Invalid refresh token"));
        }

        let user = self
            .user_repository
            .find_by_id(&jwt.sub)
            .await?
            .ok_or_else(|| Error::unauthorized("Invalid refresh token"))?;

        if jwt.token_version != user.token_version {
            return Err(Error::unauthorized("Invalid refresh token"));
        }

        let generate_auth_use_case = GenerateAuthUseCase {
            jwt_encoder: Arc::clone(&self.jwt_encoder),
        };

        generate_auth_use_case.execute(&GenerateAuthUseCaseParams {
            user_id: user.id,
            role: user.role,
            token_version: user.token_version,
        })
    }
}
