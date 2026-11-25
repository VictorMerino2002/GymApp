use std::{sync::Mutex, time::Duration};

use async_trait::async_trait;

use crate::{
    application::ports::{FileStoragePort, PresignedUpload},
    domain::value_objects::Error,
};

pub struct PresignCall {
    pub key: String,
    pub content_type: String,
    pub content_length: u64,
    pub expires_in: Duration,
}

#[derive(Default)]
pub struct FakeFileStorage {
    pub calls: Mutex<Vec<PresignCall>>,
}

#[async_trait]
impl FileStoragePort for FakeFileStorage {
    async fn presign_upload(
        &self,
        key: &str,
        content_type: &str,
        content_length: u64,
        expires_in: Duration,
    ) -> Result<PresignedUpload, Error> {
        self.calls.lock().unwrap().push(PresignCall {
            key: key.to_string(),
            content_type: content_type.to_string(),
            content_length,
            expires_in,
        });

        Ok(PresignedUpload {
            url: format!("https://bucket.s3.amazonaws.com/{key}?signature=fake"),
            headers: vec![("content-type".to_string(), content_type.to_string())],
        })
    }

    fn public_url(&self, key: &str) -> String {
        format!("https://cdn.example.com/{key}")
    }
}
