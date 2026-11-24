use std::sync::Arc;

use crate::{
    application::{
        ports::{JwtEncoderPort, repositories::UserRepository},
        use_cases::auth::{
            GenerateAuthUseCase, GenerateAuthUseCaseParams, GenerateAuthUseCaseResponse,
        },
    },
    domain::value_objects::Error,
};

pub struct LoginUseCase {
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
    pub user_repository: Arc<dyn UserRepository>,
}

pub struct LoginUseCaseParams {
    pub username: String,
}

impl LoginUseCase {
    pub async fn execute(
        &self,
        params: LoginUseCaseParams,
    ) -> Result<GenerateAuthUseCaseResponse, Error> {
        let user = self
            .user_repository
            .find_by_username(&params.username)
            .await?
            .ok_or_else(|| Error::unauthorized("Invalid credentials"))?;

        let generate_auth_use_case = GenerateAuthUseCase {
            jwt_encoder: Arc::clone(&self.jwt_encoder),
        };

        generate_auth_use_case.execute(&GenerateAuthUseCaseParams {
            user_id: user.id,
            role: user.role,
        })
    }
}
