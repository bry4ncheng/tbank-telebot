pub mod config;
pub mod controllers;
pub mod enums;
pub mod repositories;
pub mod services;
pub mod models;

use std::sync::Arc;
use crate::config::AppConfig;
use clap::Parser;
use crate::repositories::tbank_repository::TBankRepository;
use crate::services::telegram_service::TelegramService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Initialize environment
    get_app_config();
    dotenv::dotenv().ok();
    let app_config = Arc::new(AppConfig::parse());

    //Instantiate service
    let tbank_repo = TBankRepository::new(app_config.tbank_url.clone());
    let telegram_service = TelegramService::new(
        &app_config.teloxide_token,
        tbank_repo
    );

    let cloned_telegram_service = telegram_service.clone();

    tokio::spawn(async move {
        let _ = cloned_telegram_service
            .listen_and_reply()
            .await;
    });

    let _ = controllers::server::serve(
        app_config,
        telegram_service
    ).await.unwrap();

    Ok(())
}
pub fn get_app_config() -> Arc<AppConfig> {
    dotenv::dotenv().ok();
    let app_config = Arc::new(AppConfig::parse());
    app_config
}
