use axum::Router;

use crate::infrastructure::bootstrap::AppState;

mod application;
mod domain;
mod infrastructure;

async fn router() -> Router {
    Router::new()
}

#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    let router = router().await;
    let app_state = AppState::default().await;

    let app_state = if std::env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
        app_state.set_lambda_api(router)
    } else {
        app_state.set_local_api("0.0.0.0:8080", router)
    };

    app_state.run_api().await
}
