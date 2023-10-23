use crate::enums::telegram::Command;
use reqwest::Client;
use teloxide::prelude::ResponseResult;
use teloxide::{prelude::Requester, repls::CommandReplExt, types::Message, Bot};
use tracing::{info};
use crate::repositories::tbank_repository::TBankRepository;

#[derive(Clone)]
pub struct TelegramService {
    bot: Bot,
    tbank_repository: TBankRepository
}

impl TelegramService {
    pub fn new(
        bot_token: &String,
        tbank_repo: TBankRepository,
    ) -> Self {
        let reqwest_client = Client::new();
        let bot = Bot::with_client(bot_token, reqwest_client);
        Self {
            bot,
            tbank_repository: tbank_repo
        }
    }

    pub async fn listen_and_reply(self) {
        Command::repl(self.bot.clone(), move |bot, msg, cmd| {
            TelegramService::answer(self.clone(), bot, msg, cmd)
        })
            .await;
    }

    pub async fn answer(
        self,
        bot: Bot,
        msg: Message,
        cmd: Command,
    ) -> ResponseResult<()> {
        match cmd {
            Command::Start => {
                //Check if user has been authenticated via GET
                let text = msg.text().unwrap().to_string();
                //split from /start by "" then take the second index
                let key = text.split(" ").collect::<Vec<&str>>()[1];
                info!("Received message: {:?}", key);
                let _ = bot
                    .send_message(msg.chat.id, "hello".to_string())
                    .await;
                Ok(())
            }
        }
    }
}
