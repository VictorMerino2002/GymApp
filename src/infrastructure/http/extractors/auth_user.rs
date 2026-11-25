use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
    response::{IntoResponse, Response},
};

use crate::{
    domain::value_objects::{Error, Role, TokenType},
    infrastructure::{bootstrap::AppState, http::value_objects::ApiResponse},
};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub role: Role,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth_user) = parts.extensions.get::<AuthUser>() {
            return Ok(auth_user.clone());
        }

        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(|| unauthorized("Missing or malformed Authorization header"))?;

        let jwt = app_state
            .jwt_encoder
            .decode(token)
            .map_err(|_| unauthorized("Invalid token"))?;

        if jwt.token_type != TokenType::Access {
            return Err(unauthorized("Invalid token"));
        }

        let user = app_state
            .user_repository
            .find_by_id(&jwt.sub)
            .await
            .map_err(|error| ApiResponse::<()>::default(Err(error)).into_response())?
            .ok_or_else(|| unauthorized("Invalid token"))?;

        if jwt.token_version != user.token_version {
            return Err(unauthorized("Invalid token"));
        }

        let auth_user = Self {
            user_id: user.id,
            role: user.role,
        };
        parts.extensions.insert(auth_user.clone());

        Ok(auth_user)
    }
}

fn unauthorized(message: &str) -> Response {
    ApiResponse::<()>::default(Err(Error::unauthorized(message))).into_response()
}
