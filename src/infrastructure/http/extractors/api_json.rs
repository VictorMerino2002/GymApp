use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;

use crate::{domain::value_objects::Error, infrastructure::http::value_objects::ApiResponse};

pub struct ApiJson<T>(pub T);

impl<T, S> FromRequest<S> for ApiJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(rejection_to_response(rejection)),
        }
    }
}

fn rejection_to_response(rejection: JsonRejection) -> Response {
    let error = Error::new(rejection.status(), &rejection.body_text());
    ApiResponse::<()>::default(Err(error)).into_response()
}
