
use std::sync::Arc;

use crate::config::AppConfig;
use crate::enums::telegram::Command;
use crate::models::customer::OnBoardCustomerData;
use crate::models::{Error, CustomerRequest};
use crate::models::authentication::{RequestOTP, ServiceLoginOtpResponse};
use clap::Parser;
use reqwest::Client;
use teloxide::prelude::ResponseResult;
use teloxide::{prelude::Requester, types::Message, Bot};
use tracing::info;
use crate::repositories::tbank_repository::TBankRepository;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, Me
    },
    utils::command::BotCommands,
};
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

        for chunk_options in options.chunks(3) {
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
            app_config.tbank_url.clone()
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
                                            bot.send_message(msg.chat.id, "Sorry that your session for sign up is gone. Please try again.").await?;
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
                                                        let invest_key: String = format!("{}:{}",msg.chat.id.to_string(), "AutoInvest");
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
                                            bot.send_message(msg.chat.id, "Sorry that your session for sign up is gone. Please try again.").await?;
                                            TelegramService::send_start( bot, msg.chat.id.to_string()).await?; 
                                        },
                                    }
                                }
                                &"Sign Up" =>{
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
                app_config.tbank_url.clone()
            );
            let mut account_number = "".to_owned();
            if action.contains("Account") && action != "Remove Account"{
                account_number = action.split(" ").last().unwrap().to_string();
                action = "Account".to_owned();
            }
            info!("GOT TBANK");
            match &action.as_str() {
                &"Login" =>{
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
                &"Sign Up" =>{
                    // Push to redis user state to invalidate 
                    let text = "Let's start with your chosen username";
                    // Edit text of the message to which the buttons were attached
                    let keyboard = Self::make_keyboard(["Cancel".to_owned()].to_vec());
                    if let Some(Message { id, chat, .. }) = q.message {
                        let action_key = format!("{}:{}", chat.id.to_string(), "action");
                        let _ = redis_repo.clone().remove_data_in_redis(&action_key).await;
                        let _ = redis_repo.set_data_in_redis(&action_key,"Sign Up".to_owned(), true).await;
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
                                let result_details = tbank_repo.clone().get_customer_details(request_data.clone()).await;
                                match result_details{
                                    Ok(data) => {
                                        let to_send = OnBoardCustomerData{
                                            service_name: "onboardCustomer".to_owned(),
                                            ic_number: data.content.service_response.cdm_customer.certificate.certificate_no.unwrap_or("".to_owned()),
                                            family_name: data.content.service_response.cdm_customer.family_name,
                                            given_name: data.content.service_response.cdm_customer.given_name,
                                            date_of_birth: data.content.service_response.cdm_customer.date_of_birth,
                                            gender: data.content.service_response.cdm_customer.profile.gender.unwrap_or("".to_owned()),
                                            occupation: data.content.service_response.cdm_customer.profile.occupation.unwrap_or("".to_owned()),
                                            street_address: data.content.service_response.cdm_customer.address.street_address1.unwrap_or("".to_owned()),
                                            city: data.content.service_response.cdm_customer.address.city.unwrap_or("".to_owned()),
                                            state: data.content.service_response.cdm_customer.address.state.unwrap_or("".to_owned()),
                                            country: data.content.service_response.cdm_customer.address.country.unwrap_or("".to_owned()),
                                            postal_code: data.content.service_response.cdm_customer.address.postal_code.unwrap_or("".to_owned()),
                                            country_code: data.content.service_response.cdm_customer.cellphone.country_code.unwrap_or("".to_owned()),
                                            mobile_number: data.content.service_response.cdm_customer.cellphone.phone_number.unwrap_or("".to_owned()),
                                            preferred_user_id: request_data.user_id,
                                            currency: "".to_owned(),
                                            bank_id: data.content.service_response.cdm_customer.profile.bank_id.unwrap_or("".to_owned()),
                                        };
                                        let onboard_result = tbank_repo.onboard_customer(to_send).await;
                                        match onboard_result{
                                            Ok(onboard_reply) => {
                                                // let j: String = serde_json::to_string(&onboard_reply).unwrap();
                                                info!("DATA: {}", onboard_reply);
                                            },
                                            Err(_) => todo!(),
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
                    // Delete user AutoInvest
                    if let Some(Message { id, chat, .. }) = q.message {
                        let full_key: String = format!("{}:{}",chat.id.to_string(), "AutoInvest");
                        let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                        let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), "Enable AutoInvest".to_owned(),].to_vec());
                        bot.edit_message_text(chat.id, id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
                    } else if let Some(id) = q.inline_message_id {
                        TelegramService::send_start( bot, id.to_string()).await?;
                    }
                }
                &"Back" =>{
                    if let Some(Message { id, chat, .. }) = q.message {
                        let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), "Enable AutoInvest".to_owned(),].to_vec());
                        bot.edit_message_text(chat.id, id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
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
                                let full_key: String = format!("{}:{}",data.user_id, "AutoInvest");
                                let _ = redis_repo.clone().remove_data_in_redis(&full_key).await;
                                let _ = redis_repo.clone().set_data_in_redis(&full_key, account_number.clone(), false).await;
                                bot.edit_message_text(chat.id, id, format!("You have chosen: {}\nThank you!", account_number)).await?;
                                let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), "Update AutoInvest".to_owned(),].to_vec());
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
                &"Enable AutoInvest" =>{
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
                                                let temp =format!("{} - {}%\n", one.account_id.to_string(), one.interest_rate);
                                                options.push( format!("Account: {}", one.account_id.clone()));
                                                full_text = format!("{}{}", full_text, temp);
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
                &"Update AutoInvest" =>{
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
                                let invest_key: String = format!("{}:{}",data.user_id.to_string(), "AutoInvest");
                                let invest_account = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                                    Ok(r) => r,
                                    Err(_) => "".to_string(),
                                };
                                let account_result = tbank_repo.get_customer_accounts(data).await;
                                match account_result{
                                    Ok(accounts) => {
                                        {
                                            let mut full_text = format!("Your current AutoInvest account {}.\nPlease select one or would you like to create a new one?\n", invest_account);
                                            // let mut options = ["Back"].to_vec();
                                            let mut options = [].to_vec();

                                            for one in accounts{
                                                let temp =format!("{} - {}%\n", one.account_id.to_string(), one.interest_rate);
                                                full_text = format!("{}{}", full_text, temp);

                                                if one.account_id != invest_account{
                                                    options.push( format!("Account: {}", one.account_id.clone()));
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
                                        for one in accounts{
                                            let temp =format!("{} - {}{}\n", one.account_id, one.currency, one.balance);
                                            full_text = format!("{}{}", full_text, temp);
                                        }
                                        let keyboard = Self::make_keyboard(["Back".to_owned()].to_vec());
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
                let invest_key: String = format!("{}:{}",data.user_id, "AutoInvest");
                let has_invest = match redis_repo.clone().get_data_from_redis(&invest_key).await{
                    Ok(_) => true,
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
        let keyboard = Self::make_keyboard(["Login".to_owned(), "Sign Up".to_owned()].to_vec());
        bot.send_message(id, "Welcome to TBANK Bot! How can I help you today?").reply_markup(keyboard).await?;
        Ok(())
    }

    async fn send_logged_in_user_start(bot:Bot, id:String, has_invest:bool) -> ResponseResult<()> {
        let invest_option = if has_invest{"Enable AutoInvest".to_owned()}else{"Update AutoInvest".to_owned()};
        let keyboard = Self::make_keyboard(["Check Balance".to_owned(), "Transfer".to_owned(), "Logout".to_owned(), invest_option].to_vec());
        bot.send_message(id, "Hello! What banking service can I help you with today?").reply_markup(keyboard).await?;
        Ok(())
    }
}
