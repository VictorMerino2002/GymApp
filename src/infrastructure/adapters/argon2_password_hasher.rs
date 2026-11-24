use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;

use crate::{application::ports::PasswordHasherPort, domain::value_objects::Error};

pub struct Argon2PasswordHasher;

#[async_trait]
impl PasswordHasherPort for Argon2PasswordHasher {
    async fn hash(&self, password: &str) -> Result<String, Error> {
        let password = password.to_string();

        tokio::task::spawn_blocking(move || {
            let salt = SaltString::generate(&mut OsRng);
            Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .map(|hash| hash.to_string())
        })
        .await
        .map_err(|_| Error::internal_server("Could not hash password"))?
        .map_err(|_| Error::internal_server("Could not hash password"))
    }

    async fn verify(&self, password: &str, hash: &str) -> Result<bool, Error> {
        let password = password.to_string();
        let hash = hash.to_string();

        tokio::task::spawn_blocking(move || {
            let parsed = PasswordHash::new(&hash)
                .map_err(|_| Error::internal_server("Could not verify password"))?;

            Ok(Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok())
        })
        .await
        .map_err(|_| Error::internal_server("Could not verify password"))?
    }
}
