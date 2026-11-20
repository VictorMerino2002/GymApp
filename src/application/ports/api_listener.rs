use std::error::Error;

use async_trait::async_trait;

#[async_trait(?Send)]
pub trait ApiListener: Send + Sync {
    async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
}
