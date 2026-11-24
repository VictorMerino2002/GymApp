use std::sync::Arc;

use crate::{
    application::{
        ports::{JwtEncoderPort, PasswordHasherPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::value_objects::Error,
};

pub struct LoginUseCase {
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
    pub user_repository: Arc<dyn UserRepository>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
}

pub struct LoginUseCaseParams {
    pub username: String,
    pub password: String,
}

impl LoginUseCase {
    pub async fn execute(
        &self,
        params: LoginUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        let user = self
            .user_repository
            .find_by_username(&params.username)
            .await?;

        let Some((user, password_hash)) =
            user.and_then(|user| user.password_hash.clone().map(|hash| (user, hash)))
        else {
            self.password_hasher.hash(&params.password).await?;
            return Err(Error::unauthorized("Invalid credentials"));
        };

        if !self
            .password_hasher
            .verify(&params.password, &password_hash)
            .await?
        {
            return Err(Error::unauthorized("Invalid credentials"));
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
