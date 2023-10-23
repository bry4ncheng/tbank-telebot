use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Context;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE};
use axum::http::Method;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::cors;
use tower_http::cors::CorsLayer;
use tracing::info;
use crate::config::AppConfig;
use crate::controllers::health;
use crate::services::register_service::ServiceRegister;
use crate::services::telegram_service::TelegramService;

pub async fn serve(
    config: Arc<AppConfig>,
    telegram_service: TelegramService
) -> anyhow::Result<()> {
    // Register Services to be used in handlers
    let services = ServiceRegister::new(
        config.clone(),
        telegram_service
    ).await;

    let app = Router::new()
        .nest("/", health::router())
        .with_state(services) // Inject services into handlers as state
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_methods([
                        Method::GET,
                        Method::POST,
                        Method::OPTIONS,
                    ])
                    .allow_headers([AUTHORIZATION, ACCEPT, COOKIE, CONTENT_TYPE])
                    .allow_origin(cors::Any)// In a real application, you should validate the `Origin` header.
            ),
        );

    info!("Starting server at port {}", 3000);

    axum::Server::bind(&"0.0.0.0:3000".parse::<SocketAddr>()?)
        .serve(app.into_make_service())
        .await
        .context("Error starting server")

}