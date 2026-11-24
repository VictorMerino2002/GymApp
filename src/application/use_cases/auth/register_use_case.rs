use std::sync::Arc;

use crate::{
    application::{
        ports::{JwtEncoderPort, PasswordHasherPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::{
        entities::{User, validate_password, validate_username},
        value_objects::Error,
    },
};

pub struct RegisterUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
    pub password_hasher: Arc<dyn PasswordHasherPort>,
}

pub struct RegisterUseCaseParams {
    pub username: String,
    pub password: String,
}

impl RegisterUseCase {
    pub async fn execute(
        &self,
        params: RegisterUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        validate_username(&params.username)?;

        validate_password(&params.password)?;

        let existing_user = self
            .user_repository
            .find_by_username(&params.username)
            .await?;

        if existing_user.is_some() {
            return Err(Error::conflict("Username already taken"));
        }

        let password_hash = self.password_hasher.hash(&params.password).await?;
        let new_user = User::new(&params.username, password_hash);
        self.user_repository.create(&new_user).await?;

        let generate_auth_use_case = GenerateAuthUseCase {
            jwt_encoder: Arc::clone(&self.jwt_encoder),
        };

        generate_auth_use_case.execute(&GenerateAuthUseCaseParams {
            user_id: new_user.id,
            role: new_user.role,
            token_version: new_user.token_version,
        })
    }
}
