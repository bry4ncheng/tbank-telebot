use serde::{Deserialize, Serialize};
use crate::models::Error;
#[derive(Serialize, Deserialize, Clone)]
pub struct RequestOTP {
    #[serde(rename = "serviceName")]
    pub service_name: String,
    #[serde(rename = "userID")]
    pub user_id: String,
    #[serde(rename = "PIN")]
    pub pin: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReplyLoginCustomer {
    #[serde(rename = "CustomerID")]
    pub customer_id: Option<String>,
    #[serde(rename = "BankID")]
    pub bank_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerData {
    #[serde(rename = "AccountID")]
    pub account_id: String,
    #[serde(rename = "branchID")]
    pub branch_id: Option<String>,
    #[serde(rename = "PIN")]
    pub pin: String,
    #[serde(rename = "CustomerID")]
    pub customer_id: String,
}
#[derive(Serialize, Deserialize)]
pub struct ServiceLoginOtpResponse {
    #[serde(rename = "Login_OTP_Authenticate-Response")]
    pub login_otp_response: ReplyLoginCustomer,
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: Error
}

#[derive(Serialize, Deserialize)]
pub struct ReplyOnboardCustomer {
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: Error,
    #[serde(rename = "CustomerDetails")]
    pub customer_details: Option<CustomerData>
}