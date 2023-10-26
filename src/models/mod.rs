use serde::{Deserialize, Serialize};

use self::authentication::ReplyLoginCustomer;

pub mod transaction;
pub mod customer;
pub mod authentication;
#[derive(Serialize, Deserialize)]
pub struct TBankResponse<T> {
    #[serde(rename = "Content")]
    pub content: ServiceResponseBody<T>
}
#[derive(Serialize, Deserialize)]
pub struct ServiceResponseBody<T> {
    #[serde(rename = "ServiceResponse")]
    pub service_response: ServiceResponseHeader<T>
}
#[derive(Serialize, Deserialize)]
pub struct ServiceResponseHeader<T> {
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: T
}

#[derive(Serialize, Deserialize)]
pub struct TBankLoginResponse {
    #[serde(rename = "Content")]
    pub content: ServiceLoginResponseBody
}

#[derive(Serialize, Deserialize)]
pub struct ServiceLoginResponseBody {
    #[serde(rename = "ServiceResponse")]
    pub service_login_response: ServiceLoginOtpResponse,

}
#[derive(Serialize, Deserialize)]
pub struct ServiceLoginOtpResponse {
    #[serde(rename = "Login_OTP_Authenticate-Response")]
    pub login_otp_response: ReplyLoginCustomer,
    #[serde(rename = "ServiceRespHeader")]
    pub service_response_header: Error
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    #[serde(rename = "ErrorText")]
    pub error_text: Option<String>,
    #[serde(rename = "ErrorDetails")]
    pub error_details: Option<String>,
    #[serde(rename = "GlobalErrorID")]
    pub global_error_id: Option<String>,
}