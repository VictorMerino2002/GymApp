use std::sync::Arc;

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};

use crate::{
    domain::value_objects::{Error, Role},
    infrastructure::{
        bootstrap::AppState,
        http::{extractors::AuthUser, value_objects::ApiResponse},
    },
};

pub struct AdminUser {
    pub user_id: String,
}

impl FromRequestParts<Arc<AppState>> for AdminUser {
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut Parts,
        app_state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = AuthUser::from_request_parts(parts, app_state).await?;

        if auth_user.role != Role::Admin {
            let error = Error::forbidden("Admin role required");
            return Err(ApiResponse::<()>::default(Err(error)).into_response());
        }

        Ok(Self {
            user_id: auth_user.user_id,
        })
    }
}
