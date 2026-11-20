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
    let app_state = AppState::default().await.set_lambda_api(router);
    // .set_local_api("0.0.0.0:8080", router);
    app_state.run_api().await
}
