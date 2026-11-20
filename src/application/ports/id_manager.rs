use async_trait::async_trait;

#[async_trait]
pub trait IdManager: Send + Sync {
    async fn generate(&self) -> String;
}
