use axum::{
    extract::{FromRequestParts, Query, rejection::QueryRejection},
    http::request::Parts,
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;

use crate::{domain::value_objects::Error, infrastructure::http::value_objects::ApiResponse};

pub struct ApiQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for ApiQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request_parts(parts, state).await {
            Ok(Query(value)) => Ok(Self(value)),
            Err(rejection) => Err(rejection_to_response(rejection)),
        }
    }
}

fn rejection_to_response(rejection: QueryRejection) -> Response {
    let error = Error::new(rejection.status(), &rejection.body_text());
    ApiResponse::<()>::default(Err(error)).into_response()
}
