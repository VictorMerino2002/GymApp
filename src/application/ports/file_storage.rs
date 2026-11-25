use std::time::Duration;

use async_trait::async_trait;

use crate::domain::value_objects::Error;

pub struct PresignedUpload {
    pub url: String,
    pub headers: Vec<(String, String)>,
}

#[async_trait]
pub trait FileStoragePort: Send + Sync {
    async fn presign_upload(
        &self,
        key: &str,
        content_type: &str,
        content_length: u64,
        expires_in: Duration,
    ) -> Result<PresignedUpload, Error>;
    fn public_url(&self, key: &str) -> String;
}
