use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;

use crate::{application::ports::PasswordHasherPort, domain::value_objects::Error};

#[derive(Default)]
pub struct FakePasswordHasher {
    pub hash_calls: AtomicUsize,
}

#[async_trait]
impl PasswordHasherPort for FakePasswordHasher {
    async fn hash(&self, password: &str) -> Result<String, Error> {
        self.hash_calls.fetch_add(1, Ordering::SeqCst);
        Ok(format!("hash:{password}"))
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, Error> {
        Ok(hash == format!("hash:{password}"))
    }
}
