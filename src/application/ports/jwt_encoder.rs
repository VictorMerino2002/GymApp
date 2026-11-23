use crate::domain::value_objects::{Error, Jwt};

pub trait JwtEncoderPort: Send + Sync {
    fn encode(&self, jwt: &Jwt) -> Result<String, Error>;
    fn decode(&self, token: &str) -> Result<Jwt, Error>;
}
