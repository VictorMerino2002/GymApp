use std::error::Error;

use async_trait::async_trait;
use axum::Router;

use crate::application::ports::ApiListener;

pub struct LambdaApiListener {
    router: Router,
}

impl LambdaApiListener {
    pub fn new(router: Router) -> Self {
        Self { router }
    }
}

#[async_trait(?Send)]
impl ApiListener for LambdaApiListener {
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        tracing_subscriber::fmt()
            .with_max_level(lambda_http::tracing::Level::INFO)
            .with_target(false)
            .with_ansi(false)
            .without_time()
            .init();

        lambda_http::run(self.router.clone()).await
    }
}
