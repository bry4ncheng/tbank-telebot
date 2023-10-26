use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct RequestOTP {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(rename = "userID")]
    pub user_id: String,
    #[serde(rename = "PIN")]
    pub pin: String,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(rename = "userID")]
    pub user_id: String,
    #[serde(rename = "PIN")]
    pub pin: String,
    #[serde(rename = "OTP")]
    pub otp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyOTP {
    #[serde(rename = "ErrorText")]
    pub error_text: Option<String>,
    #[serde(rename = "GlobalErrorID")]
    pub global_error_id: Option<String>,
    #[serde(rename = "ErrorDetails")]
    pub error_details: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyLoginCustomer {
    #[serde(rename = "CustomerID")]
    pub customer_id: Option<String>,
    #[serde(rename = "BankID")]
    pub bank_id: Option<String>,
}
