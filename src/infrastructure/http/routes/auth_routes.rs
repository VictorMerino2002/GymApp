use axum::{Json, response::IntoResponse};

pub struct LoginReq {
    username: String,
}

pub fn login(Json(payload): Json<LoginReq>) -> impl IntoResponse {}
