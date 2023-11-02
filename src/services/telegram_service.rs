
use std::sync::Arc;

use crate::config::AppConfig;
use crate::enums::telegram::Command;
use crate::models::transaction::{TransferBody, AddBeneficiaryBody};
use crate::models::{Error, CustomerRequest};
use crate::models::authentication::{RequestOTP, ServiceLoginOtpResponse};
use clap::Parser;
use rand::Rng;
use reqwest::Client;
use teloxide::prelude::ResponseResult;
use teloxide::{prelude::Requester, types::Message, Bot};
use tracing::{info, warn};
use crate::repositories::tbank_repository::TBankRepository;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, Me
    },
    utils::command::BotCommands,
};
use teloxide::types::InputFile;
use crate::enums::beneficiary::BeneficiaryEnum;
use crate::models::customer::HistoricalMonthlyBalanceBody;
use crate::repositories::redis_repository::RedisRepository;


#[derive(Clone)]
pub struct TelegramService {
    bot: Bot,
}

impl TelegramService {
    pub fn new(
        bot_token: &String,
    ) -> Self {
        let reqwest_client = Client::new();
        let bot = Bot::with_client(bot_token, reqwest_client);
        Self {
            bot,
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
    fn make_keyboard(options:Vec<String>) -> InlineKeyboardMarkup {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

        for chunk_options in options.chunks(1) {
            let row = chunk_options
                .iter()
                .map(|option| InlineKeyboardButton::callback(option.to_owned(), option.to_owned()))
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
        //Instantiate service
        let app_config: Arc<AppConfig> = Arc::new(AppConfig::parse());
        info!("READ APP CONFIG");

        let redis_repo = RedisRepository::new(
            app_config.redis_url.clone()
        ).await;
        info!("GOT REDIS");
        
        let tbank_repo = TBankRepository::new(
            app_config.tbank_url.clone(),
            app_config.chart_generator_url.clone()
        );
        info!("GOT TBANK");
        if let Some(text) = msg.text() {
            match BotCommands::parse(text, me.username()) {
                Ok(Command::Help) => {
                    // Just send the description of all commands.
                    bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
                }
                Ok(Command::Start) => {
                    // Create a list of buttons and send them.
                    TelegramService::to_send_correct_start(bot, msg, redis_repo.clone(), true).await?;            
                }
                Err(_) => {
                    // Check redis state on what step he is on or if he has any valid state.
                    let action_key: String = format!("{}:{}",msg.chat.id.to_string(), "action");

                    match redis_repo.clone().get_data_from_redis(&action_key).await {
                        Ok(result) => {
                            match &result.as_str() {
                                &"Add Ben Desc" => {
                                    let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                                    let my_int: i32 = msg.id.to_string().parse().unwrap();
                                    bot.edit_message_text(msg.chat.id, teloxide::types::MessageId(my_int-1), "Please wait ...").await?;
                                    let full_key: String = format!("{}:{}",msg.chat.id.to_string(), "AddBen");
                                    let temp = redis_repo.clone().get_data_from_redis(&full_key).await.unwrap();
                                    let mut add_ben_data=  serde_json::from_str::<AddBeneficiaryBody>(&temp).unwrap();
                                    let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                    add_ben_data.description = text.to_string();
                                    let full_key: String = format!("{}:{}",msg.chat.id.to_string(), "LoginCred");
                                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                                        match result {
                                            Ok(login_cred) => {
                                                let mut data: CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                                data.service_name = "addBeneficiary".to_owned();
                                                let result = tbank_repo.add_beneficiary(data, add_ben_data).await;
                                                if let Ok(status) = result{
                                                    if status.contains("invocation successful"){
                                                        bot.delete_message(msg.chat.id, msg.id).await?;
                                                        bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                                        bot.send_message(msg.chat.id,  "Beneficiary has been added").await?;
                                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), true).await?;

                                                    }else{
                                                        bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                                                    }
                                                }else{
                                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                                    TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                                                }
                                            },
                                            Err(_) => {
                                                bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                                            },
                                        }
                                }
                                &"Add Beneficiary" => {
                                    let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                                    let temp  = AddBeneficiaryBody{
                                        account_id: text.to_string(),
                                        description: "".to_owned(),
                                    };
                                    bot.delete_message(msg.chat.id, msg.id).await?;
                                    let my_int: i32 = msg.id.to_string().parse().unwrap();
                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                    let full_key: String = format!("{}:{}",msg.chat.id.to_string(), "AddBen");
                                    let temp_string: String =  serde_json::to_string(&temp).unwrap();
                                    let _ = redis_repo.clone().set_data_in_redis(&full_key, temp_string, false).await;
                                    let _ = redis_repo.clone().set_data_in_redis(&action_key, "Add Ben Desc".to_owned(), false).await;

                                    let keyboard = Self::make_keyboard(["Back".to_owned()].to_vec());
                                    bot.send_message(msg.chat.id,  "Label for the account?").reply_markup(keyboard).await?;
                                }
                                &"Amount" => {
                                    let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                                    let amount = text.trim().parse::<f64>();
                                    let my_int: i32 = msg.id.to_string().parse().unwrap();
                                    bot.edit_message_text(msg.chat.id, teloxide::types::MessageId(my_int-1), "Please wait ...").await?;
                                    if let Ok(a) = amount{
                                        bot.delete_message(msg.chat.id, msg.id).await?;

                                        let full_key: String = format!("{}:{}",msg.chat.id.to_string(), "LoginCred");
                                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                                        match result {
                                            Ok(login_cred) => {
                                                let mut data: CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                                data.service_name = "getCustomerAccounts".to_owned();
                                                let account_result = tbank_repo.get_customer_accounts(data).await;
                                                match account_result{
                                                    Ok(accounts) => {
                                                        let mut vec_kb: Vec<String> = vec![];
                                                        for one in accounts {
                                                            if one.balance.parse::<f64>().unwrap_or(0.0) > a{
                                                                let acc = format!("Transfer From {}", one.account_id).to_string();
                                                                vec_kb.push(acc);
                                                            }
                                                        }
                                                        if vec_kb.len() > 0{
                                                            let tx_key: String = format!("{}:{}", msg.chat.id.to_string(), "Transfer");
                                                            let temp = redis_repo.clone().get_data_from_redis(&tx_key).await.unwrap();
                                                            let mut tx_body =  serde_json::from_str::<TransferBody>(&temp).unwrap();
                                                            tx_body.transaction_amount = format!("{:.2}", a);
                                                            let _ = redis_repo.clone().remove_data_in_redis(&tx_key).await;
                                                            let tx_body_string =  serde_json::to_string(&tx_body).unwrap();
                                                            let _ = redis_repo.clone().set_data_in_redis(&tx_key, tx_body_string, false).await;
                                                            vec_kb.push("Back".to_owned());
                                                            bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                                            let keyboard = Self::make_keyboard(vec_kb);
                                                            bot.send_message(msg.chat.id,  "Which account would you like to use?").reply_markup(keyboard).await?;
                                                        }else{
                                                            TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;                                                        }
                                                    }
                                                    Err(_) => {
                                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                                                    },
                                                }
                                            },
                                            Err(_) => {
                                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                                            },
                                        }
                
                                    }else{
                                        bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                        TelegramService::to_send_correct_start( bot, msg, redis_repo.clone(), false).await?; 
                                    }
                                }
                                &"Login" =>{
                                    let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                                    let _ = redis_repo.clone().set_data_in_redis(&action_key,"Login:PIN".to_owned(), true).await;
                                    let part_key: String = format!("{}:{}",msg.chat.id.to_string(), "LoginStep");
                                    let _ = redis_repo.clone().remove_data_in_redis(&part_key).await;
                                    let empty = RequestOTP{ 
                                        user_id: text.to_string(),
                                        service_name: "requestOTP".to_string(),
                                        pin: "".to_string(),
                                    };
                                    let j = serde_json::to_string(&empty).unwrap();
                                    let _ = redis_repo.clone().set_data_in_redis(&part_key,j, true).await;
                                    let keyboard = Self::make_keyboard(["Cancel".to_owned()].to_vec());
                                    let my_int: i32 = msg.id.to_string().parse().unwrap();  
                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int)).await?;
                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                    bot.send_message(msg.chat.id, "Please key in your PIN").reply_markup(keyboard).await?;
                                }
                                &"Login:PIN"=>{
                                    bot.delete_message(msg.chat.id, msg.id).await?;
                                    let my_int: i32 = msg.id.to_string().parse().unwrap();
                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                    let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                                    let part_key: String = format!("{}:{}",msg.chat.id.to_string(), "LoginStep");
                                    match redis_repo.clone().get_data_from_redis(&part_key).await{
                                        Ok(partial_result) => {
                                            let _ = redis_repo.clone().remove_data_in_redis(&part_key).await;
                                            let mut data:RequestOTP = serde_json::from_str(&partial_result).unwrap();
                                            data.pin = text.to_string();
                                            bot.send_message(msg.chat.id, "Checking your credentials....").await?;
                                            match tbank_repo.request_otp(data.clone()).await{
                                                Ok(reply) => {
                                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int+1)).await?;
                                                    let reply_otp = reply.content.service_response.service_response_header as Error;
                                                    match reply_otp.error_details{
                                                        Some(status) => {
                                                            info!("{}", status);
                                                            if status != "success" {
                                                                bot.send_message(msg.chat.id, "Sorry It seems like we could not authenticate you. Please try again.").await?;
                                                                TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                                            }else{
                                                                let partial_login_request = CustomerRequest{ 
                                                                    service_name: "loginCustomer".to_owned(), 
                                                                    user_id: data.user_id, 
                                                                    pin: data.pin, 
                                                                    otp: "".to_owned() 
                                                                };
                                                                let j = serde_json::to_string(&partial_login_request).unwrap();
                                                                let _ = redis_repo.clone().set_data_in_redis(&part_key,j, true).await;
                                                                let _ = redis_repo.clone().set_data_in_redis(&action_key,"Login:OTP".to_owned(), true).await;
                                                                let keyboard = Self::make_keyboard(["Cancel".to_owned()].to_vec());
                                                                bot.send_message(msg.chat.id, "Please key in your OTP").reply_markup(keyboard).await?;
                                                            }   
                                                        },
                                                        None => {
                                                            bot.send_message(msg.chat.id, "Sorry It seems like we could not authenticate you. Please try again.").await?;
                                                            TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                                        }
                                                    }
                                                                                 
                                                },
                                                Err(_) => {
                                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int+1)).await?;
                                                    bot.send_message(msg.chat.id, "Sorry It seems like we could not authenticate you. Please try again.").await?;
                                                    TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                                },
                                            }
                                        },
                                        Err(_) => {
                                            bot.send_message(msg.chat.id, "Sorry that your session is gone. Please try again.").await?;
                                            TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                        },
                                    }
                                }
                                &"Login:OTP"=>{
                                    bot.delete_message(msg.chat.id, msg.id).await?;
                                    let my_int: i32 = msg.id.to_string().parse().unwrap();
                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int-1)).await?;
                                    let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                                    let part_key: String = format!("{}:{}",msg.chat.id.to_string(), "LoginStep");
                                    match redis_repo.clone().get_data_from_redis(&part_key).await{
                                        Ok(partial_result) => {
                                            let _ = redis_repo.clone().remove_data_in_redis(&part_key).await;
                                            let mut data:CustomerRequest = serde_json::from_str(&partial_result).unwrap();
                                            data.otp = text.to_string();
                                            bot.send_message(msg.chat.id, "Logging In ....").await?;
                                            match tbank_repo.login_customer(data.clone()).await{
                                                Ok(reply) => {
                                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int+1)).await?;
                                                    let response = reply.content.service_response as ServiceLoginOtpResponse;
                                                    if response.service_response_header.error_details.unwrap() != "Success".to_string() {
                                                        bot.send_message(msg.chat.id, "Sorry It seems like we could not authenticate you. Please try again.").await?;
                                                        TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                                    }else{
                                                        data.otp = "999999".to_string();
                                                        let j = serde_json::to_string(&data).unwrap();
                                                        let full_key: String = format!("{}:{}",msg.chat.id.to_string(), "LoginCred");
                                                        let _ = redis_repo.clone().set_data_in_redis(&full_key,j, false).await;
                                                        let invest_key: String = format!("{}:{}",data.user_id.clone(), "MicroInvest");
                                                        let has_invest = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                                                            Ok(_) => true,
                                                            Err(_) => false,
                                                        };
                                                        TelegramService::send_logged_in_user_start( bot, msg.chat.id.to_string(), has_invest).await?; 
                                                    }                                
                                                },
                                                Err(_) => {
                                                    bot.delete_message(msg.chat.id, teloxide::types::MessageId(my_int+1)).await?;
                                                    bot.send_message(msg.chat.id, "Sorry It seems like we could not authenticate you. Please try again.").await?;
                                                    TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                                },
                                            }
                                        },
                                        Err(_) => {
                                            bot.send_message(msg.chat.id, "Sorry that your session for is gone. Please try again.").await?;
                                            TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                        },
                                    }
                                }
                                _ => {
                                    TelegramService::to_send_correct_start(bot, msg, redis_repo.clone(), false).await?;            
                                }
                            }
                        },
                        Err(_) => {
                            bot.send_message(msg.chat.id, "Command not found!").await?;
                        },
                    };
                }
            }
        }
    
        Ok(())
    }

    async fn callback_handler(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
        if let Some(mut action) = q.data {
            bot.answer_callback_query(q.id).await?;
            //Instantiate service
            let app_config: Arc<AppConfig> = Arc::new(AppConfig::parse());
            info!("READ APP CONFIG");

            let redis_repo = RedisRepository::new(
                app_config.redis_url.clone()
            ).await;
            info!("GOT REDIS");
            
            let tbank_repo = TBankRepository::new(
                app_config.tbank_url.clone(),
                app_config.chart_generator_url.clone()
            );
            let mut account_number = "".to_owned();
            let mut percentage_to_invest = "2".to_owned();

            if action.contains("Reselect") {
                if q.message.is_some() {
                    let msg = q.message.clone().unwrap();
                    let chat = msg.clone().chat; 
                    let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                    let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                    match result {
                        Ok(login_cred) => {
                            let data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                            let full_key: String = format!("{}:{}",data.user_id, "MicroInvest");
                            action = "Enable MicroInvest".to_owned();
                            let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                        }
                        Err(_) => {
                            action = "".to_owned();
                        },
                    }
                }

            } else if action.contains("Balance History") {
                account_number = action.split(" ").nth(2).unwrap().to_string();
                action = "Chart".to_owned();
            }
            else if action.contains("%"){
                percentage_to_invest = action.replace("%", "");
                action = "Percentage".to_owned();

            } else if action.contains("Account") && action != "Remove Account"{
                account_number = action.split(" ").last().unwrap().to_string();
                action = "Account".to_owned();
            } else if action.contains("Transfer To") {
                account_number = action.split(" ").last().unwrap().to_string();
                action = "Amount".to_owned();
            }else if action.contains("Transfer From"){
                account_number = action.split(" ").last().unwrap().to_string();
                action = "TransferFrom".to_owned();
            }
            
            info!("GOT TBANK");
            match &action.as_str() {
                &"Amount" => {
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        // TODO: Transfer To String
                        let num = rand::thread_rng().gen_range(u64::MIN..u64::MAX);

                        let tx_body = TransferBody{
                            account_from: "".to_owned(),
                            account_to: account_number,
                            transaction_amount: "".to_owned(),
                            transaction_reference_number: format!("{}", num),
                            narrative: "".to_owned(),
                        };
                        let tx_key: String = format!("{}:{}", chat.id.to_string(), "Transfer");
                        let full_key: String = format!("{}:{}", chat.id.to_string(), "action");
                        let _ = redis_repo.clone().set_data_in_redis(&full_key, "Amount".to_owned(), false).await;
                        let _ = redis_repo.clone().set_data_in_redis(&tx_key, serde_json::to_string(&tx_body).unwrap(), false).await;
                        let keyboard = Self::make_keyboard(["Back".to_owned()].to_vec());
                        bot.edit_message_text(chat.id, id, "How much do you want to transfer?").reply_markup(keyboard).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Login" => {
                    // Push to redis user state to invalidate 
                    let text = "Please key in your username";
                    // Edit text of the message to which the buttons were attached
                    let keyboard = Self::make_keyboard(["Cancel".to_owned()].to_vec());
                    if let Some(Message { id, chat, .. }) = q.message {
                        let action_key = format!("{}:{}", chat.id.to_string(), "action");
                        let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                        let _ = redis_repo.set_data_in_redis(&action_key,"Login".to_owned(), true).await;
                        bot.edit_message_text(chat.id, id, text).reply_markup(keyboard).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Cancel" =>{
                    // Delete user state to invalidate 
                    if let Some(Message { id, chat, .. }) = q.message {
                        let action_key = format!("{}:{}", chat.id.to_string(), "action");
                        let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                        bot.delete_message(chat.id, id).await?;
                        TelegramService::send_start( bot, chat.id.to_string()).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Add Beneficiary" => {
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        let full_key: String = format!("{}:{}", chat.id.to_string(), "action");
                        let _ = redis_repo.clone().set_data_in_redis(&full_key, "Add Beneficiary".to_owned(), false).await;
                        let keyboard = Self::make_keyboard(["Back".to_owned()].to_vec());
                        bot.edit_message_text(chat.id, id, "Key in account number to add?").reply_markup(keyboard).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Transfer" =>{
                    // Delete user state to invalidate 
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id; 
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut data: CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                data.service_name = "getBeneficiaryList".to_owned(); 
                                let beneficiaries = match tbank_repo.get_beneficiaries(data, BeneficiaryEnum::OTHER).await {
                                    Ok(ben) => ben,
                                    Err(e) => {
                                        warn!("Something went wrong while getting beneficiaries : {}", e);
                                        TelegramService::to_send_correct_start(bot.clone(), msg.clone(), redis_repo.clone(), false).await?;
                                        vec![]
                                    }
                                };

                                let mut vec_kb: Vec<String> = vec![];
                                for ben in beneficiaries {
                                    let acc = format!("Transfer To {} {}", ben.description, ben.account_id).to_string();
                                    vec_kb.push(acc);
                                }

                                vec_kb.push("Add Beneficiary".to_owned());
                                vec_kb.push("Back".to_owned());
                                // TODO: Add Beneficiary
                                let keyboard = Self::make_keyboard(vec_kb);
                                bot.edit_message_text(chat.id, id, "Where would you like to transfer to?").reply_markup(keyboard).await?;
                            }
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot.clone(), msg.clone(), redis_repo.clone(), false).await?;
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot.clone(), id.to_string()).await?;
                    }
                }
                &"TransferFrom" => {
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        bot.edit_message_text(chat.id, id, "Please wait ...").await?;
                        let tx_key: String = format!("{}:{}", msg.chat.id.to_string(), "Transfer");
                        let temp = redis_repo.clone().get_data_from_redis(&tx_key).await.unwrap();
                        let mut tx_body =  serde_json::from_str::<TransferBody>(&temp).unwrap();
                        tx_body.account_from = account_number;
                        tx_body.narrative = "Simple Transfer".to_owned();
                        let _ = redis_repo.clone().remove_data_in_redis(&tx_key).await;
                        let _ = redis_repo.clone().set_data_in_redis(&tx_key, serde_json::to_string(&tx_body).unwrap(), false).await;
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                let invest_key: String = format!("{}:{}",data.user_id, "MicroInvest");
                                let acct = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                                    Ok(acct) => if acct != ""{Some(acct)}else{None},
                                    Err(_) => None,
                                };
                                info!("{:?} --data??", acct);

                                if let Some(acct) = acct {
                                    if acct != tx_body.account_from{
                                        let full_key: String = format!("{}:{}",data.user_id, "Percentage");
                                        match redis_repo.clone().get_data_from_redis(&full_key).await{
                                            Ok(percentage_str) => {
                                                info!("{:?} --data??", percentage_str);
                                                let percentage = percentage_str.parse::<f64>().unwrap();
                                                let temp = tx_body.transaction_amount.parse::<f64>().unwrap();
                                                let to_invest = temp * (percentage/100.0);
                                                let total: f64 = temp+to_invest;
                                                info!("{:?} --data??", total);
                                                let mut data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                                data.service_name = "getCustomerAccounts".to_owned();
                                                let account_result = tbank_repo.get_customer_accounts(data).await;
                                                let is_enough = match account_result{
                                                    Ok(accounts) => {   
                                                        let mut to_return = false;                
                                                        for one in accounts{
                                                            if one.account_id == tx_body.account_from{
                                                                to_return =  one.balance.parse::<f64>().unwrap() >= total;
                                                                info!("{:?} --data??", to_return);
                                                                break;
                                                            }
                                                        }
                                                        to_return
                                                    }
                                                    Err(_) => {
                                                        false
                                                    },
                                                };
                                                if is_enough {
                                                    let keyboard: InlineKeyboardMarkup = Self::make_keyboard(["Confirm".to_owned(), "Back".to_owned()].to_vec());
                                                    bot.edit_message_text(chat.id, id, format!("SUMMARY\nTransferring to: {}\nTransferring from {}\nAmount: ${:.2}\nMicro-Invest amount: ${:.2}", tx_body.account_to, tx_body.account_from, temp, to_invest)).reply_markup(keyboard).await?;        
                                                }else{
                                                    TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                                }
                                            },  
                                            Err(_) => {
                                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                            }                                   
                                        }
                                    }else{
                                        let keyboard: InlineKeyboardMarkup = Self::make_keyboard(["Confirm".to_owned(), "Back".to_owned()].to_vec());
                                        bot.edit_message_text(chat.id, id, format!("SUMMARY\nTransferring to: {}\nTransferring from {}\nAmount: ${}", tx_body.account_to, tx_body.account_from, tx_body.transaction_amount)).reply_markup(keyboard).await?;    
                                    }

                                }else{
                                    let keyboard: InlineKeyboardMarkup = Self::make_keyboard(["Confirm".to_owned(), "Back".to_owned()].to_vec());
                                    bot.edit_message_text(chat.id, id, format!("SUMMARY\nTransferring to: {}\nTransferring from {}\nAmount: ${}", tx_body.account_to, tx_body.account_from, tx_body.transaction_amount)).reply_markup(keyboard).await?;
                                }
                            }
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                        
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Confirm" =>{
                    if q.message.is_some() {
                        
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        bot.edit_message_text(chat.id, id, "Please wait ...").await?;        

                        let tx_key: String = format!("{}:{}", msg.chat.id.to_string(), "Transfer");
                        let temp: String = redis_repo.clone().get_data_from_redis(&tx_key).await.unwrap();
                        let mut tx_body: TransferBody =  serde_json::from_str::<TransferBody>(&temp).unwrap();
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                let invest_key: String = format!("{}:{}",data.user_id, "MicroInvest");
                                let acct = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                                    Ok(acct) => if acct != ""{Some(acct)}else{None},
                                    Err(_) => None,
                                };
                                let percent_key: String = format!("{}:{}",data.user_id, "Percentage");
                                let percentage_str = match redis_repo.clone().get_data_from_redis(&percent_key).await{
                                    Ok(percent) => if percent != ""{Some(percent)}else{None},
                                    Err(_) => None,
                                };
                                data.service_name = "creditTransfer".to_owned();
                                let r =tbank_repo.clone().transfer(data.clone(), tx_body.clone()).await;
                                if let Ok(status) = r{
                                    if status.contains("invocation successful"){
                                        if let Some(acct) = acct {
                                            if acct != tx_body.account_from{
                                                let percentage = percentage_str.unwrap().parse::<f64>().unwrap();
                                                let temp = tx_body.transaction_amount.parse::<f64>().unwrap();
                                                let to_invest = temp * (percentage/100.0);
                                                tx_body.account_to = acct;
                                                tx_body.transaction_amount = format!("{:.2}", to_invest);
                                                tx_body.narrative = "Micro-Invest".to_owned();
                                                let other_r =tbank_repo.clone().transfer(data.clone(), tx_body.clone()).await;
                                                if let Ok(other_status) = other_r{
                                                    if other_status.contains("invocation successful"){
                                                        bot.edit_message_text(chat.id, id, "The transfer has been done").await?;        
                                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), true).await?;   
                                                    }else{
                                                        bot.edit_message_text(chat.id, id, "The transfer has been done except for your Micro Invest").await?;        
                                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), true).await?;            
                                                    }
                                                }else{
                                                    bot.edit_message_text(chat.id, id, "The transfer has been done except for your Micro Inve   st").await?;        
                                                    TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), true).await?;            
                                                }
                                            }else{
                                                bot.edit_message_text(chat.id, id, "The transfer has been done").await?;        
                                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), true).await?;            
                                            }
        
                                        }else{
                                            bot.edit_message_text(chat.id, id, "The transfer has been done").await?;        
                                            TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), true).await?;            
                                        }
                                    }
                                    else{
                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                    }
                                }else{
                                    TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                }
                            }
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Create" =>{
                    // Delete user state to invalidate 
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id; 
                        bot.edit_message_text(chat.id, id, "Please wait we are creating your new account...").await?;
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut request_data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                request_data.service_name = "getCustomerDetails".to_owned();
                                let full_key: String = format!("{}:{}",request_data.user_id, "MicroInvest");
                                let result_details: Result<crate::models::TBankResponse<crate::models::customer::GetCustomerDetails>, anyhow::Error> = tbank_repo.clone().get_customer_details(request_data.clone()).await;
                                match result_details{
                                    Ok(data) => {
                                        request_data.service_name = "openDepositAccount".to_owned();
                                        request_data.pin = "1".to_owned();
                                        request_data.otp = "".to_owned();
                                        request_data.user_id = data.content.service_response.cdm_customer.certificate.certificate_no.unwrap();
                                        let open_result = tbank_repo.clone().create_account(request_data.clone()).await;
                                        match open_result{
                                            Ok(account_id) => {
                                                if account_id != "null"{
                                                    let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                                    let _ = redis_repo.clone().set_data_in_redis(&full_key, account_id.clone(), false).await;
                                                    bot.edit_message_text(chat.id, id, format!("We have created: {}", account_id)).await?;
                                                    let keyboard: InlineKeyboardMarkup = Self::make_keyboard(["2%".to_owned(), "5%".to_owned(), "10%".to_owned(), "Reselect account".to_owned()].to_vec());
                                                    bot.edit_message_text(chat.id, id, "What percentage of a transaction would you like to be added to your chosen account?").reply_markup(keyboard).await?;
                                                }else{
                                                    bot.edit_message_text(chat.id, id, "Failed creating the account.").await?;
                                                    TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                                }
                                            },
                                            Err(_) => {
                                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                            },
                                        }
                                    },
                                    Err(_) => {
                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                    },
                                }
                            }
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Logout" =>{
                    // Delete user creds
                    if let Some(Message { id, chat, .. }) = q.message {
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                        bot.delete_message(chat.id, id).await?;
                        TelegramService::send_start( bot, chat.id.to_string()).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Remove Account" =>{
                    // Delete user MicroInvest
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;   
                         let full_key: String = format!("{}:{}", msg.chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(data_string) => {      
                                let data:CustomerRequest = serde_json::from_str(&data_string).unwrap();
                                let full_key: String = format!("{}:{}",data.user_id.to_string(), "MicroInvest");
                                let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                let full_key: String = format!("{}:{}",data.user_id.to_string(), "Percentage");
                                let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                let keyboard: InlineKeyboardMarkup = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), "Enable MicroInvest".to_owned(),].to_vec());
                                bot.edit_message_text(chat.id, id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
                            },
                            Err(_) => {
                                TelegramService::send_start( bot, msg.chat.id.to_string()).await?;
                            },
                        }
                        
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Back" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;    
                        let add_ben_key: String = format!("{}:{}",msg.chat.id.to_string(), "AddBen");
                        let action_key = format!("{}:{}", chat.id.to_string(), "action");
                        let tx_key: String = format!("{}:{}", chat.id.to_string(), "Transfer");
                        let _ = redis_repo.clone().remove_data_in_redis(&tx_key).await;
                        let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                        let _ = redis_repo.clone().remove_data_in_redis(&add_ben_key).await;
                        let full_key: String = format!("{}:{}", msg.chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(data_string) => {      
                                let data:CustomerRequest = serde_json::from_str(&data_string).unwrap();
                                let invest_key: String = format!("{}:{}",data.user_id, "MicroInvest");
                                info!("{}", invest_key);
                                let has_invest = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                                    Ok(acct) => if acct != ""{true}else{false},
                                    Err(_) => false,
                                };
                                let invest_option = if has_invest{"Update MicroInvest".to_owned()}else{"Enable MicroInvest".to_owned()};
                                let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), invest_option].to_vec());
                                bot.edit_message_text(chat.id, id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
                            },
                            Err(_) => {
                                TelegramService::send_start( bot, msg.chat.id.to_string()).await?;
                            },
                        }
                        
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Account" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;    
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                let full_key: String = format!("{}:{}",data.user_id, "MicroInvest");
                                let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                let _ = redis_repo.clone().set_data_in_redis(&full_key, account_number.clone(), false).await;
                                bot.edit_message_text(chat.id, id, format!("You have chosen: {}", account_number)).await?;
                                let keyboard: InlineKeyboardMarkup = Self::make_keyboard(["2%".to_owned(), "5%".to_owned(), "10%".to_owned(), "Reselect account".to_owned()].to_vec());
                                bot.edit_message_text(chat.id, id, "What percentage of a transaction would you like to be added to your chosen account?").reply_markup(keyboard).await?;
                            }
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Percentage" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;    
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                let full_key: String = format!("{}:{}",data.user_id, "Percentage");
                                let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                let _ = redis_repo.clone().set_data_in_redis(&full_key, percentage_to_invest.clone(), false).await;
                                let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), "Update MicroInvest".to_owned()].to_vec());
                                bot.edit_message_text(chat.id, id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
        
                            }
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Enable MicroInvest" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        bot.edit_message_text(chat.id, id, "Please wait ...").await?;
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                data.service_name = "getCustomerAccounts".to_owned();
                                let account_result = tbank_repo.get_customer_accounts(data).await;
                                match account_result{
                                    Ok(accounts) => {
                                        if accounts.len() == 1 {
                                            let full_text = "You only have one account.\nPlease open a new account".to_owned();
                                            let keyboard = Self::make_keyboard(["Create".to_owned(), "Back".to_owned()].to_vec());
                                            bot.edit_message_text(chat.id, id, full_text).reply_markup(keyboard).await?;
                                        }else{
                                            let mut full_text = "You have more than one account.\nPlease select one or would you like to create a new one?\n".to_owned();
                                            // let mut options = ["Back"].to_vec();
                                            let mut options = [].to_vec();

                                            for one in accounts{
                                                if one.product_id == "101"{
                                                    let temp =format!("{} - {}%\n", one.account_id.to_string(), one.interest_rate);
                                                    options.push( format!("Account: {}", one.account_id.clone()));
                                                    full_text = format!("{}{}", full_text, temp);
                                                }
                                            }
                                            options.push("Create".to_string());
                                            options.push("Back".to_string());
                                            let keyboard = Self::make_keyboard(options);
                                            bot.edit_message_text(chat.id, id, full_text).reply_markup(keyboard).await?;
                                        }
                                    }
                                    Err(_) => {
                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                    },
                                }
                            },
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Update MicroInvest" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        bot.edit_message_text(chat.id, id, "Please wait ...").await?;
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                data.service_name = "getCustomerAccounts".to_owned();
                                let invest_key: String = format!("{}:{}",data.user_id.to_string(), "MicroInvest");
                                let invest_account = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                                    Ok(r) => r,
                                    Err(_) => "".to_string(),
                                };
                                let account_result = tbank_repo.get_customer_accounts(data).await;
                                match account_result{
                                    Ok(accounts) => {
                                        {
                                            let mut full_text = format!("Your current MicroInvest account {}.\nPlease select one or would you like to create a new one?\n", invest_account);
                                            // let mut options = ["Back"].to_vec();
                                            let mut options = [].to_vec();

                                            for one in accounts{
                                                if one.product_id == "101"{
                                                    let temp =format!("{} - {}%\n", one.account_id.to_string(), one.interest_rate);
                                                    full_text = format!("{}{}", full_text, temp);

                                                    if one.account_id != invest_account{
                                                        options.push( format!("Account: {}", one.account_id.clone()));
                                                    }
                                                }
                                            }
                                            options.push("Remove Account".to_string());
                                            options.push("Back".to_string());
                                            let keyboard = Self::make_keyboard(options);
                                            bot.edit_message_text(chat.id, id, full_text).reply_markup(keyboard).await?;
                                        }
                                    }
                                    Err(_) => {
                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                    },
                                }
                            },
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Check Balance" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        bot.edit_message_text(chat.id, id, "Please wait ...").await?;
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut data:CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                data.service_name = "getCustomerAccounts".to_owned();
                                let account_result = tbank_repo.get_customer_accounts(data).await;
                                match account_result{
                                    Ok(accounts) => {
                                        let mut full_text = "Your Account Balance is:\n".to_string();
                                        let mut vec_acc = vec![];

                                        for one in accounts {
                                            let temp =format!("{} - {}{}\n", one.account_id, one.currency, one.balance);
                                            full_text = format!("{}{}", full_text, temp);
                                            vec_acc.push(format!("View Account {} Balance History", one.account_id));
                                        }
                                        vec_acc.push("Back".to_owned());
                                        let keyboard = Self::make_keyboard(vec_acc);
                                        bot.edit_message_text(chat.id, id, full_text).reply_markup(keyboard).await?;
                                    }
                                    Err(_) => {
                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                                    },
                                }
                            },
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;            
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }

                }
                &"Chart" =>{
                    if q.message.is_some() {
                        let msg = q.message.unwrap();
                        let chat = msg.clone().chat;
                        let id = msg.clone().id;
                        bot.edit_message_text(chat.id, id, "Please wait ...").await?;
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "LoginCred");
                        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
                        match result {
                            Ok(login_cred) => {
                                let mut data: CustomerRequest = serde_json::from_str(&login_cred).unwrap();
                                data.service_name = "getMonthlyBalanceTrend".to_owned();
                                let content = HistoricalMonthlyBalanceBody {
                                    account_id: account_number.clone(),
                                    //Default to 6
                                    num_months: "6".to_string(),
                                };
                                let monthly_balance_result = tbank_repo.clone().get_monthly_balance_trend(data,  content).await;
                                match monthly_balance_result{
                                    Ok(accounts) => {
                                        let chart = tbank_repo.clone().get_balance_chart(accounts).await.unwrap();
                                        let full_text = format!("{} balance over the past 6 months", account_number.clone());
                                        bot.delete_message(chat.id, msg.id).await?;
                                        let png = InputFile::memory(chart);
                                        bot.send_photo(chat.id, png).await?;
                                        let keyboard = Self::make_keyboard(["Back".to_owned()].to_vec());
                                        bot.send_message(chat.id, full_text).reply_markup(keyboard).await?;
                                    }
                                    Err(_) => {
                                        TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                                    },
                                }
                            },
                            Err(_) => {
                                TelegramService::to_send_correct_start(bot, msg.clone(), redis_repo.clone(), false).await?;
                            },
                        }
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }

                }
                _ => {
                    //Invalidate user state
                    if let Some(Message { id, chat, .. }) = q.message {
                        let action_key = format!("{}:{}", chat.id.to_string(), "action");
                        let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                        bot.delete_message(chat.id, id).await?;
                        TelegramService::send_start( bot, chat.id.to_string()).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
            }
        }
    
        Ok(())
    }


    async fn to_send_correct_start(bot:Bot, msg: Message, redis_repo:RedisRepository, is_start: bool) -> ResponseResult<()> {
        let full_key: String = format!("{}:{}", msg.chat.id.to_string(), "LoginCred");
        let result = redis_repo.clone().get_data_from_redis(&full_key).await;
        match result {
            Ok(data_string) => {
                if !is_start{
                    bot.delete_message(msg.chat.id, msg.id).await?;
                    bot.send_message(msg.chat.id, "Sorry something went wrong. Please try again.").await?;
                }
                let data:CustomerRequest = serde_json::from_str(&data_string).unwrap();
                let invest_key: String = format!("{}:{}",data.user_id, "MicroInvest");
                info!("{}", invest_key);
                let has_invest = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                    Ok(acct) => if acct != ""{true}else{false},
                    Err(_) => false,
                };
                TelegramService::send_logged_in_user_start( bot, msg.chat.id.to_string(), has_invest).await?; 
            },
            Err(_) => {
                if !is_start{
                    bot.send_message(msg.chat.id, "Sorry something went wrong. Please try again.").await?;
                }
                TelegramService::send_start( bot, msg.chat.id.to_string()).await?;
            },
        }
        Ok(())
    }

    async fn send_start(bot:Bot, id:String) -> ResponseResult<()> {
        let keyboard = Self::make_keyboard(["Login".to_owned()].to_vec());
        bot.send_message(id, "Welcome to TBANK Bot! How can I help you today?").reply_markup(keyboard).await?;
        Ok(())
    }

    async fn send_logged_in_user_start(bot:Bot, id:String, has_invest:bool) -> ResponseResult<()> {
        let invest_option = if has_invest{"Update MicroInvest".to_owned()}else{"Enable MicroInvest".to_owned()};
        let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), invest_option].to_vec());
        bot.send_message(id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
        Ok(())
    }
}
