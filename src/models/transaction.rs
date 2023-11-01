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

#[derive(Serialize, Deserialize)]
pub struct BeneficiaryList {
    //Shows successful messages as well.. strange
    #[serde(rename = "BeneficiaryList")]
    pub beneficiary_list: Beneficiaries,
}

#[derive(Serialize, Deserialize)]
pub struct Beneficiaries {
    //Shows successful messages as well.. strange
    #[serde(rename = "Beneficiary")]
    pub beneficiary: Vec<Beneficiary>
}

#[derive(Serialize, Deserialize)]
pub struct Beneficiary {
    //Shows successful messages as well.. strange
    #[serde(rename = "AccountID")]
    pub account_id: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Currency")]
    pub currency: String,
    #[serde(rename = "BeneficiaryID")]
    pub beneficiary_id: String
}

#[derive(Serialize, Deserialize)]
pub struct AddBeneficiaryBody {
    #[serde(rename = "AccountID")]
    pub account_id: String,
    #[serde(rename = "Description")]
    pub description: String
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TransferBody {
    #[serde(rename = "accountFrom")]
    pub account_from: String,
    #[serde(rename = "accountTo")]
    pub account_to: String,
    #[serde(rename = "transactionAmount")]
    pub transaction_amount: String,
    #[serde(rename = "transactionReferenceNumber")]
    pub transaction_reference_number: String,
    #[serde(rename = "narrative")]
    pub narrative: String
}

#[derive(Serialize, Deserialize)]
pub struct TransferResponse {
    pub status_text: String,
    pub post_balance: Option<String>,
    pub pre_balance: Option<String>,
    pub transaction_amount: String,
    pub transaction_reference_number: Option<String>,
}