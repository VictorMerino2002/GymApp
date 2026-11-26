use std::time::Duration;

use axum::http::{
    Method,
    header::{AUTHORIZATION, CONTENT_TYPE},
};
use tower_http::cors::{AllowOrigin, CorsLayer};

const PREFLIGHT_MAX_AGE: Duration = Duration::from_secs(600);

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(AllowOrigin::mirror_request())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([CONTENT_TYPE, AUTHORIZATION])
        .max_age(PREFLIGHT_MAX_AGE)
}
