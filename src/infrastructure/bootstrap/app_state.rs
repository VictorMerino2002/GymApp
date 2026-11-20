use std::error::Error;

use axum::Router;

use crate::{
    application::ports::{ApiListener, JwtEncoderPort, UserRepository},
    infrastructure::adapters::{
        JwtEncoder, LambdaApiListener, LocalApiListener, repositories::DynamodbUserRepository,
    },
};

pub struct AppState {
    pub user_repository: Box<dyn UserRepository>,
    pub jwt_encoder: Box<dyn JwtEncoderPort>,
    pub api_listener: Option<Box<dyn ApiListener>>,
}

impl AppState {
    pub async fn default() -> Self {
        let aws_config = aws_config::load_from_env().await;
        let dynamodb = aws_sdk_dynamodb::Client::new(&aws_config);
        let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE env var not set");

        let user_repository = DynamodbUserRepository::new(dynamodb, users_table);
        let jwt_encoder = JwtEncoder::new("".to_string());

        Self {
            user_repository: Box::new(user_repository),
            jwt_encoder: Box::new(jwt_encoder),
            api_listener: None,
        }
    }

    pub fn set_local_api(mut self, address: impl Into<String>, router: Router) -> Self {
        self.api_listener = Some(Box::new(LocalApiListener::new(address, router)));
        self
    }

    pub fn set_lambda_api(mut self, router: Router) -> Self {
        self.api_listener = Some(Box::new(LambdaApiListener::new(router)));
        self
    }

    pub async fn run_api(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        match &self.api_listener {
            Some(api_listener) => api_listener.run().await,
            None => Err(
                "API listener not configured, call set_local_api or set_lambda_api first".into(),
            ),
        }
    }
}
