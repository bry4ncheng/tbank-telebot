use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub struct DepositRequest {
    #[serde(rename = "serviceName")]
    service_name: String,
    #[serde(rename = "userID")]
    user_id: String,
    #[serde(rename = "PIN")]
    pin: String,
    //Not required
    #[serde(rename = "OTP")]
    otp: String,
    #[serde(rename = "accountID")]
    account_id: String,
    amount: String,
    narrative: String,
}

#[derive(Serialize, Deserialize)]
pub struct DepositResponse {
    //Shows successful messages as well.. strange
    #[serde(rename = "ErrorText")]
    error_text: String,
    #[serde(rename = "GlobalErrorID")]
    global_error_id: String,
    #[serde(rename = "ErrorDetails")]
    error_details: String,
    #[serde(rename = "BalanceAfter")]
    balance_after: String,
    #[serde(rename = "BalanceBefore")]
    balance_before: String,
    #[serde(rename = "TransactionID")]
    transaction_id: String,
}