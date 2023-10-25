
use crate::enums::telegram::Command;
use reqwest::Client;
use teloxide::prelude::ResponseResult;
use teloxide::{prelude::Requester, types::Message, Bot};
use crate::repositories::tbank_repository::TBankRepository;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, Me
    },
    utils::command::BotCommands,
};


#[derive(Clone)]
pub struct TelegramService {
    bot: Bot,
    tbank_repository: TBankRepository,
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
        let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(Self::message_handler))
        .branch(Update::filter_callback_query().endpoint(Self::callback_handler));
        // .branch(Update::filter_inline_query().endpoint(Self::inline_query_handler));

        Dispatcher::builder(self.bot.clone(), handler).enable_ctrlc_handler().build().dispatch().await;
    }

    /// Creates a keyboard made by buttons in a big column.
    fn make_keyboard(options:Vec<&str>) -> InlineKeyboardMarkup {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

        for chunk_options in options.chunks(3) {
            let row = chunk_options
                .iter()
                .map(|&option| InlineKeyboardButton::callback(option.to_owned(), option.to_owned()))
                .collect();

            keyboard.push(row);
        }

        InlineKeyboardMarkup::new(keyboard)
    }
    
    async fn message_handler(
        bot: Bot,
        msg: Message,
        me: Me,
    ) -> ResponseResult<()>  {
        if let Some(text) = msg.text() {
            match BotCommands::parse(text, me.username()) {
                Ok(Command::Help) => {
                    // Just send the description of all commands.
                    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
                }
                Ok(Command::Start) => {
                    // Create a list of buttons and send them.
                    let _ = Self::send_start(bot, msg.chat.id.to_string());
                }
                Err(_) => {
                    // Check redis state on what step he is on or if he has any valid state.
                    bot.send_message(msg.chat.id, "Command not found!").await?;
                }
            }
        }
    
        Ok(())
    }

    async fn callback_handler(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
        if let Some(action) = q.data {
            bot.answer_callback_query(q.id).await?;

            match &action.as_str() {
                &"Login" =>{
                    // Push to redis user state to invalidate 
                    let text = "Please key in your username";
                    // Edit text of the message to which the buttons were attached
                    let keyboard = Self::make_keyboard(["Cancel"].to_vec());
                    if let Some(Message { id, chat, .. }) = q.message {
                        bot.edit_message_text(chat.id, id, text).reply_markup(keyboard).await?;
                    } else if let Some(id) = q.inline_message_id {
                        bot.edit_message_text_inline(id, text).reply_markup(keyboard).await?;
                    }
                }
                &"Sign Up" =>{
                    // Push to redis user state to invalidate 
                    let text = "Let's start with your chosen username";
                    // Edit text of the message to which the buttons were attached
                    let keyboard = Self::make_keyboard(["Cancel"].to_vec());
                    if let Some(Message { id, chat, .. }) = q.message {
                        bot.edit_message_text(chat.id, id, text).reply_markup(keyboard).await?;
                    } else if let Some(id) = q.inline_message_id {
                        bot.edit_message_text_inline(id, text).reply_markup(keyboard).await?;
                    }
                }
                &"Cancel" =>{
                    // Delete user state to invalidate 
                    let text = "User has cancel the action";
                    if let Some(Message { id, chat, .. }) = q.message {
                        bot.delete_message(chat.id, id).await?;
                        let _ = Self::send_start(bot, chat.id.to_string());
                    } else if let Some(id) = q.inline_message_id {
                        bot.edit_message_text_inline(id.clone(), text).await?;
                        let _ = Self::send_start(bot, id);
                    }
                }
                _ => {
                    //Invalidate user state
                    let text = "User has done an invalid action";
                    if let Some(Message { id, chat, .. }) = q.message {
                        bot.delete_message(chat.id, id).await?;
                        let _ = Self::send_start(bot, chat.id.to_string());
                    } else if let Some(id) = q.inline_message_id {
                        bot.edit_message_text_inline(id.clone(), text).await?;
                        let _ = Self::send_start(bot, id);
                    }
                }
            }
        }
    
        Ok(())
    }

    async fn send_start(bot:Bot, id:String){
        let keyboard = Self::make_keyboard(["Login", "Sign Up"].to_vec());
        let _ = bot.send_message(id, "Welcome to TBANK Bot! How can I help you today?").reply_markup(keyboard).await;
    }
}
