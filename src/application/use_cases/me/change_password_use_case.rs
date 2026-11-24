use std::sync::Arc;

use crate::{
    application::{
        ports::{JwtEncoderPort, PasswordHasherPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::{entities::validate_password, value_objects::Error},
};

pub struct ChangePasswordUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
}

pub struct ChangePasswordUseCaseParams {
    pub user_id: String,
    pub current_password: String,
    pub new_password: String,
}

impl ChangePasswordUseCase {
    pub async fn execute(
        &self,
        params: ChangePasswordUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        validate_password(&params.new_password)?;

        let user = self
            .user_repository
            .find_by_id(&params.user_id)
            .await?
            .ok_or_else(|| Error::unauthorized("User not found"))?;

        let current_hash = user
            .password_hash
            .as_deref()
            .ok_or_else(|| Error::unauthorized("Invalid credentials"))?;

        if !self
            .password_hasher
            .verify(&params.current_password, current_hash)
            .await?
        {
            return Err(Error::unauthorized("Invalid credentials"));
        }

        let new_hash = self.password_hasher.hash(&params.new_password).await?;
        let token_version = self
            .user_repository
            .update_password(&user.id, &new_hash)
            .await?;

        let generate_auth_use_case = GenerateAuthUseCase {
            jwt_encoder: Arc::clone(&self.jwt_encoder),
        };

        generate_auth_use_case.execute(&GenerateAuthUseCaseParams {
            user_id: user.id,
            role: user.role,
            token_version,
        })
    }
}
