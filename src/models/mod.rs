use serde::{Deserialize, Serialize};
pub mod transaction;
pub mod customer;
pub mod authentication;
pub mod chart;

#[derive(Debug, Serialize, Deserialize)]
pub struct TBankResponse<T> {
    #[serde(rename = "Content")]
    pub content: ServiceResponseBody<T>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponseBody<T> {
    #[serde(rename = "ServiceResponse")]
    pub service_response: T
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceResponseHeader<T> {
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: T
}


#[derive(Serialize, Deserialize, Clone)]
pub struct CustomerRequest {
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
pub struct Error {
    #[serde(rename = "ErrorText")]
    pub error_text: Option<String>,
    #[serde(rename = "ErrorDetails")]
    pub error_details: Option<String>,
    #[serde(rename = "GlobalErrorID")]
    pub global_error_id: Option<String>,
}