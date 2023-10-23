use serde::{Deserialize, Serialize};

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
    pub service_response: T
}

#[derive(Serialize, Deserialize)]
pub struct Error {
    #[serde(rename = "ErrorText")]
    pub error_text: String,
    #[serde(rename = "ErrorDetails")]
    pub error_details: Option<String>,
    #[serde(rename = "GlobalErrorID")]
    pub global_error_id: String,
}