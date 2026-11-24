use std::sync::Arc;

const MIN_PASSWORD_LENGTH: usize = 8;
const MAX_PASSWORD_LENGTH: usize = 128;

use crate::{
    application::{
        ports::{JwtEncoderPort, PasswordHasherPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::{entities::User, value_objects::Error},
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
        let password_length = params.password.chars().count();

        if !(MIN_PASSWORD_LENGTH..=MAX_PASSWORD_LENGTH).contains(&password_length) {
            return Err(Error::bad_request(&format!(
                "password must be between {MIN_PASSWORD_LENGTH} and {MAX_PASSWORD_LENGTH} characters"
            )));
        }

        let existing_user = self
            .user_repository
            .find_by_username(&params.username)
            .await?;

        if existing_user.is_some() {
            return Err(Error::bad_request(&format!(
                "User {} already exists",
                params.username
            )));
        }

        let password_hash = self.password_hasher.hash(&params.password).await?;
        let new_user = User::new(&params.username, password_hash);
        self.user_repository.save(&new_user).await?;

        let generate_auth_use_case = GenerateAuthUseCase {
            jwt_encoder: Arc::clone(&self.jwt_encoder),
        };

        generate_auth_use_case.execute(&GenerateAuthUseCaseParams {
            user_id: new_user.id,
            role: new_user.role,
        })
    }
}
