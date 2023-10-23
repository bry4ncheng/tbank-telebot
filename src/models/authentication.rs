use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct RequestOTP {
    #[serde(rename = "serviceName")]
    service_name: String,
    #[serde(rename = "userID")]
    user_id: String,
    #[serde(rename = "PIN")]
    pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyOTP {
    #[serde(rename = "ErrorText")]
    error_text: String,
    #[serde(rename = "GlobalErrorID")]
    global_error_id: String,
    #[serde(rename = "ErrorDetails")]
    error_details: String,
}
