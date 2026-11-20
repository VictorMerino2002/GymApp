use std::error::Error;

use async_trait::async_trait;
use axum::{Router, serve};

use crate::application::ports::ApiListener;

pub struct LocalApiListener {
    router: Router,
    address: String,
}

impl LocalApiListener {
    pub fn new(address: impl Into<String>, router: Router) -> Self {
        Self {
            router,
            address: address.into(),
        }
    }
}

#[async_trait(?Send)]
impl ApiListener for LocalApiListener {
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = tokio::net::TcpListener::bind(&self.address).await.unwrap();
        println!("Listening on: http://{}", self.address);
        serve(listener, self.router.clone()).await?;
        Ok(())
    }
}
