use std::sync::Arc;
use crate::config::AppConfig;
use crate::services::telegram_service::TelegramService;

#[derive(Clone)]
pub struct ServiceRegister {
    pub telegram_service: Option<TelegramService>
}

impl ServiceRegister {
    pub async fn new(
        _app_config: Arc<AppConfig>,
        telegram_service: TelegramService,
    ) -> Self {
        Self {
            telegram_service: Some(telegram_service)
        }
    }
}