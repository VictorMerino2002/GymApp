use std::sync::Arc;

use crate::{
    application::ports::JwtEncoderPort,
    domain::value_objects::{Error, Jwt, TokenType},
};

pub struct GenerateAuthUseCase {
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
}

pub struct GenerateAuthUseCaseParams {
    pub user_id: String,
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
        let access_jwt = Jwt::new(&params.user_id, TokenType::Access, 24);
        let refresh_jwt = Jwt::new(&params.user_id, TokenType::Refresh, 72);

        let access_tkn = self.jwt_encoder.encode(&access_jwt)?;
        let refresh_tkn = self.jwt_encoder.encode(&refresh_jwt)?;

        Ok(GenerateAuthUseCaseResponse {
            access: access_tkn,
            refresh: refresh_tkn,
        })
    }
}
