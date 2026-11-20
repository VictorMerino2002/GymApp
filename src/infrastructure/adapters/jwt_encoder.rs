use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

use crate::application::ports::JwtEncoderPort;
use crate::domain::value_objects::{Error, Jwt};

pub struct JwtEncoder {
    secret: String,
}

impl JwtEncoder {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl JwtEncoderPort for JwtEncoder {
    fn encode(&self, jwt: &Jwt) -> Result<String, Error> {
        encode(
            &Header::default(),
            jwt,
            &EncodingKey::from_secret(self.secret.as_bytes())
                .map_err(|_| Error::bad_request("Invalid token"))?,
        )
        .map_err(|_| Error::bad_request("Invalid token"))
    }

    fn decode(&self, token: &str) -> Result<Jwt, Error> {
        decode::<Jwt>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes())
                .map_err(|_| Error::bad_request("Ivalid token"))?,
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| Error::bad_request("Invalid token"))
    }
}
