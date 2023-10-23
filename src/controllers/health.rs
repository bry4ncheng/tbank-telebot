use axum::{Router, routing::get, http::StatusCode};

use crate::services::register_service::ServiceRegister;

pub fn router() -> Router<ServiceRegister> {
    Router::new().route("/health", get(get_health_check))
}

async fn get_health_check() -> Result<StatusCode, StatusCode> {
    Ok(StatusCode::OK)
}