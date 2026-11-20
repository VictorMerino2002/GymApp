use crate::{
    application::ports::JwtEncoderPort,
    domain::value_objects::{Error, Jwt},
};

pub struct LoginUseCase {
    pub jwt_encoder: Box<dyn JwtEncoderPort>,
}

pub struct LoginUseCaseParams {
    pub username: String,
}

pub struct LoginUseCaseResponse {
    pub access: String,
    pub refresh: String,
}

impl LoginUseCaseParams {
    pub fn new(username: impl Into<String>) -> Self {
        Self {
            username: username.into(),
        }
    }
}

impl LoginUseCase {
    pub fn execute(&self, params: LoginUseCaseParams) -> Result<String, Error> {
        let jwt = Jwt::new(&params.username);
        self.jwt_encoder.encode(&jwt)
    }
}
