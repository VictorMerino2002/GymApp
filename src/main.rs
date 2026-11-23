use std::sync::Arc;

use crate::{
    application::ports::ApiListener,
    infrastructure::{
        adapters::{LambdaApiListener, LocalApiListener},
        bootstrap::AppState,
        http::routes::router,
    },
};

mod application;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let app_state = Arc::new(AppState::default().await);
    let router = router(app_state);

    let api_listener: Box<dyn ApiListener> = if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        Box::new(LambdaApiListener::new(router))
    } else {
        Box::new(LocalApiListener::new("0.0.0.0:8080", router))
    };

    api_listener.run().await
}
