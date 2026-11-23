use std::sync::Arc;

use crate::{
    application::ports::{JwtEncoderPort, UserRepository},
    infrastructure::adapters::{JwtEncoder, repositories::DynamodbUserRepository},
};

pub struct AppState {
    pub user_repository: Arc<dyn UserRepository>,
    pub jwt_encoder: Arc<dyn JwtEncoderPort>,
}

impl AppState {
    pub async fn default() -> Self {
        let aws_config = aws_config::load_from_env().await;
        let dynamodb = aws_sdk_dynamodb::Client::new(&aws_config);
        let users_table = std::env::var("USERS_TABLE").expect("USERS_TABLE env var not set");

        let user_repository = DynamodbUserRepository::new(dynamodb, users_table);
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET env var not set");
        let jwt_encoder = JwtEncoder::new(jwt_secret);

        Self {
            user_repository: Arc::new(user_repository),
            jwt_encoder: Arc::new(jwt_encoder),
        }
    }
}
