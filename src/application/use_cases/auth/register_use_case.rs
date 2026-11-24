use std::sync::Arc;

use crate::{
    application::{
        ports::{JwtEncoderPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::{entities::User, value_objects::Error},
};

pub struct RegisterUseCase {
    pub user_repository: Arc<dyn UserRepository>,
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
}

pub struct RegisterUseCaseParams {
    pub username: String,
}

impl RegisterUseCase {
    pub async fn execute(
        &self,
        params: RegisterUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
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

        let new_user = User::new(&params.username);
        self.user_repository.save(&new_user).await?;

        let generate_auth_use_case = GenerateAuthUseCase {
            jwt_encoder: Arc::clone(&self.jwt_encoder),
        };

        generate_auth_use_case.execute(&GenerateAuthUseCaseParams {
            user_id: new_user.id,
        })
    }
}
