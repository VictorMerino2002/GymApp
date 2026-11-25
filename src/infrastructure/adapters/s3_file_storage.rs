use std::time::Duration;

use async_trait::async_trait;
use aws_sdk_s3::{Client, presigning::PresigningConfig};

use crate::{
    application::ports::{FileStoragePort, PresignedUpload},
    domain::value_objects::Error,
};

pub struct S3FileStorage {
    client: Client,
    bucket: String,
    public_base_url: String,
}

impl S3FileStorage {
    pub fn new(
        client: Client,
        bucket: impl Into<String>,
        public_base_url: impl Into<String>,
    ) -> Self {
        Self {
            client,
            bucket: bucket.into(),
            public_base_url: public_base_url.into().trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl FileStoragePort for S3FileStorage {
    async fn presign_upload(
        &self,
        key: &str,
        content_type: &str,
        content_length: u64,
        expires_in: Duration,
    ) -> Result<PresignedUpload, Error> {
        let config = PresigningConfig::expires_in(expires_in)
            .map_err(|_| Error::internal_server("Could not create upload URL"))?;
        let content_length = i64::try_from(content_length)
            .map_err(|_| Error::bad_request("size_bytes is too large"))?;

        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .content_length(content_length)
            .presigned(config)
            .await
            .map_err(|_| Error::internal_server("Could not create upload URL"))?;

        Ok(PresignedUpload {
            url: presigned.uri().to_string(),
            headers: presigned
                .headers()
                .map(|(name, value)| (name.to_string(), value.to_string()))
                .collect(),
        })
    }

    fn public_url(&self, key: &str) -> String {
        format!("{}/{key}", self.public_base_url)
    }
}
